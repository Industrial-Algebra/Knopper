# Viewport × presence-anchor design

**Status:** decision record (identity-restoration plan, Unit 4 item 1).
Decided *before* any viewport-bounded projection lands — the plan's
non-goal is explicit: no implementation until this decision exists.

## The collision

Two planned features constrain the same scene walk:

1. **Viewport-bounded projection** (performance): render only what the
   local viewport can show — cull subtrees outside the visible rect so
   layout/lowering cost tracks the *visible* scene, not the semantic
   scene. This is the natural next perf unit after Unit 1 (which made
   diffing O(n); bounding makes n = visible-n).
2. **Presence anchors** (roadmap 03): `Presence.anchor: Option<NodeId>`
   points at the semantic node a remote participant is attending to.
   Remote cursors/highlights render *at* the anchor.

The collision: a cull pass that drops offscreen subtrees will, by
default, drop the nodes that presence overlays need to attach to. A
remote cursor anchored to `node #417` inside a collapsed/offscreen
region has nowhere to render — the overlay data is alive (presence is
semantically shared) but the geometry is culled (visibility is local).

Roadmap 03's rule states the requirement precisely:

> presence is semantically shared, but visibility is local.

So the anchor's *node* may be invisible while the *participant* remains
visible (as an offscreen indicator, scrollbar marker, minimap hint, or
status line). The design must keep both facts representable.

## The options

### Option A — always-rendered annotation lane

Presence overlays never live inside the culled tree. The renderer
reserves a dedicated lane (conceptually: an overlay plane above the
scene) that the cull pass never touches. Overlays resolve their anchor
to a screen position (or an explicit offscreen marker) *after* layout,
using a lookup of laid-out node rects.

- **Pros:** the cull pass stays geometry-only and dumb — no presence
  awareness, no special cases; overlays cannot be culled by accident;
  matches roadmap 03's "render as dedicated scene nodes, likely outside
  the primary interactive focus graph".
- **Cons:** the lane needs the rect lookup (anchor → laid-out `Rect`)
  to survive culling — i.e. layout must still visit anchored nodes or
  keep a rect cache for culled-but-anchored nodes; two walks where one
  might do.

### Option B — anchor exemption pass

The cull pass consults the presence set: nodes that are the current
anchor of any roster entry (or ancestors thereof) are exempt from
culling, so overlays attach normally.

- **Pros:** overlays use ordinary scene attachment; no second plane.
- **Cons:** the cull pass now depends on `Shared` state — a geometry
  optimization coupled to collaboration semantics, the exact layering
  violation the substrate discipline exists to prevent; worst-case
  interaction with many anchors (the exemption set grows with
  participants, bounding the perf win from outside); exemption makes
  "visibility is local" false *structurally* (an offscreen anchored
  node is now rendered because a remote looks at it).

## Decision: Option A

**The annotation lane wins.** Reasons, in order of weight:

1. **Layering.** Culling must be a pure function of geometry. Option B
   couples the projection fast-path to the collaboration payload —
   the same anti-pattern as focus routed through `Shared`, rejected in
   roadmap 06.
2. **The semantics are already local.** "Visibility is local" means the
   local participant chooses how to represent an offscreen remote —
   nothing, marker, scrollbar, minimap, status. Option A's lane is
   exactly the place those representations live; Option B forces
   full-node rendering instead, collapsing the local choice.
3. **Perf honesty.** Unit 1's discipline: measure, then keep. A bounding
   pass whose worst case is "every participant anchors something
   visible-adjacent" regresses toward unbounded. The lane keeps the
   bound: cull cost tracks visible-n, lane cost tracks participants.

## Consequences (requirements for the future implementation)

- **`find_node`-style rect lookup must survive culling.** The layout
  pass (or a rect cache populated during cull decisions) must be able
  to answer "rect of node #417" even when #417 was culled from
  lowering. Cheap version: culled nodes keep their *computed* rect in
  the cache; only the render ops are skipped.
- **Anchor resolution returns an enum**, not a `Rect`:
  `AnchorPosition::Visible(Rect) | Offscreen(Direction) | Detached`.
  `Detached` = the anchored node no longer exists (scene reprojected
  away) — the overlay degrades to the roster/status rendering, never
  panics, never blocks.
- **The lane consumes the roster multivector (Unit 2 reader pattern).**
  Presence-slot annotations already derive from blade coefficients via
  `tone_census`/`presence_annotation`; offscreen indicators and status
  hints are more of the same: GA→render readers over the summed roster,
  never the typed value. The lane has *no* dependency on `Presence`
  structs — only on multivectors and rect lookups.
- **Capability seam stays orthogonal.** `gated_scene` (Unit 3) marks
  gated nodes `disabled`; culling treats `disabled` as ordinary (a
  disabled offscreen node culls like any other). The lanes compose:
  gate first (policy), then cull (geometry), then the presence lane
  renders from what policy and geometry decided.
- **Bounding boxes before culling.** The cull pass needs node rects to
  decide visibility, so layout must run (or incrementally update)
  before/beside cull — the O(n) layout from Unit 1's findings remains
  the pipeline cost center; bounding limits *lowering* and *diff*
  input sizes, not layout itself. Honest expectation-setting, same as
  the rayon reversion: the win is scoped, not magical.

## Explicitly undecided (for the implementing unit)

- Whether the rect cache is a `HashMap<NodeId, Rect>` filled during
  layout (simplest, matches `find_node` today) or a generation-indexed
  arena (faster, more code).
- Whether `AnchorPosition::Offscreen` carries enough metadata for
  scrollbar markers (needs edge distance) — cheap to add then.
- The demo surface for offscreen indicators (roadmap 03 lists five
  local rendering choices; the demo needs one).
