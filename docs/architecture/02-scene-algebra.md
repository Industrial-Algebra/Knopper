# Knopper Architecture: Scene Algebra

## Status

Draft.

## Purpose

This document defines the first-pass direction for Knopper's terminal-native scene algebra.

Knopper scenes are not DOM nodes and should not inherit browser assumptions. A scene is a structured, identity-rich, traversable representation of terminal interface intent.

A `Scene<Msg>` should support:
- layout,
- content,
- interactivity,
- style,
- focus,
- overlays,
- collaboration-aware annotations,
- diff-friendly identity.

---

## Design goals

The scene algebra should be:

- **terminal-native** rather than HTML-shaped,
- **rich enough from the start** to avoid later structural breakage,
- **identity-rich** for diffing and routing,
- **typed** with message-aware interactivity,
- **backend-independent**,
- **traversable** for layout, focus, diffing, and collaboration passes.

---

## Scene as declarative intent

A scene is not a rendered frame. It is a declarative algebra describing:
- what structure exists,
- how layout should work,
- what semantic content is present,
- what interactions are possible,
- what metadata or collaboration anchors exist.

The renderer lowers a layouted scene into backend-specific render operations.

---

## Core scene shape

A first-pass scene node likely needs:

- stable node identity,
- semantic role/kind,
- style information,
- layout hints,
- interaction bindings,
- children or content payload,
- metadata/annotations.

Conceptually:

```rust
struct Node<Msg> {
    id: NodeId,
    role: Role,
    style: Style,
    layout: LayoutHints,
    interaction: Interaction<Msg>,
    annotation: AnnotationMap,
    kind: NodeKind<Msg>,
}
```

This is illustrative only.

---

## Scene categories

Knopper should begin with a relatively rich algebra.

### 1. Structural/layout nodes
These define spatial organization.

Recommended first-pass categories:
- `Row`
- `Column`
- `Grid`
- `Stack`
- `Dock`
- `Spacer`
- `Padding`
- `Align`
- `Sized`
- `Viewport`
- `Scroll`
- `Clip`

### 2. Content nodes
These define visible terminal content.

Recommended first-pass categories:
- `Text`
- `Span`
- `Paragraph`
- `List`
- `Table`
- `Canvas`
- `Rule`
- `Border`
- `Surface`

### 3. Interactive nodes
These define semantic input targets.

Recommended first-pass categories:
- `Focusable`
- `Input`
- `Editor`
- `Selectable`
- `CommandSurface`
- `Shortcut`
- `Action`

### 4. Reactive/control nodes
These define structural dynamism.

Recommended first-pass categories:
- `Conditional`
- `Switch`
- `Dynamic`
- `Portal`
- `Annotated`

### 5. Collaboration-aware nodes
These define shared/multi-user semantics.

Recommended first-pass categories:
- `Presence`
- `RemoteCursor`
- `RemoteSelection`
- `SharedViewport`
- `ConflictOverlay`
- `SyncStatus`
- `ParticipantLayer`

---

## Identity requirements

Every scene node should be able to carry a stable `NodeId`.

### Why this matters
Stable identity supports:
- diff stability,
- focus restoration,
- event routing,
- child machine embedding,
- collaboration anchor attachment,
- renderer reuse.

### Recommendation
Node identity should be explicit, not inferred purely from tree position.

Tree position may be used as a fallback address, but it is not sufficient for long-lived semantic identity.

---

## Semantic roles

In addition to `NodeId`, scene nodes should support semantic roles or kinds.

Examples:
- header,
- footer,
- sidebar,
- editor,
- list-item,
- command-palette,
- status-line,
- participant-cursor,
- sync-indicator.

These roles help with:
- testing,
- theming,
- accessibility-like future semantics,
- debugging,
- collaboration overlays.

---

## Style model

Styles should be compositional values, not incidental renderer flags.

They likely include:
- foreground/background colors,
- text attributes,
- border style,
- emphasis/weight,
- selection/focus/hover variants,
- theme-derived defaults.

### Recommendation
Style composition should follow semigroup/monoid semantics where practical.

This allows deterministic merging of:
- inherited styles,
- node-local styles,
- theme styles,
- focus styles,
- collaboration overlays.

Karpal can help formalize this.

---

## Layout hints

Scenes should carry layout intent, not resolved layout geometry.

Possible layout hints include:
- min/max width and height,
- preferred size,
- flex/grow/shrink semantics,
- dock region,
- alignment,
- padding and margins,
- clipping behavior,
- overflow behavior,
- z-order preference.

Resolved bounds belong in the layouted scene, not the declarative scene.

---

## Interaction model

Scene nodes should encode semantic interaction targets.

These should prefer typed message descriptors over arbitrary closures.

Examples:
- `on_activate -> Msg`
- `on_focus -> Msg`
- `on_input_edit -> Msg`
- `on_select -> Msg`
- `shortcut -> Msg`

For richer interactions, routing tables or structured action descriptors may be needed.

This is preferable to storing opaque renderer-bound callbacks inside nodes.

---

## Collaboration annotations

Because collaboration is first-class, the scene algebra should support annotations that can anchor shared semantics.

Examples:
- node belongs to replicated region,
- node corresponds to shared document position,
- node can host remote cursor,
- node can host remote selection,
- node is a presence surface,
- node reflects sync health.

These annotations may be separate from visible node kinds, but the scene must carry them.

---

## Suggested intermediate representations

Knopper should distinguish at least three layers:

### 1. Declarative scene
The author- and Machine-facing tree.

### 2. Layouted scene
A scene with resolved:
- bounds,
- clipping,
- effective styles,
- z-order,
- interaction hit regions,
- collaboration overlay placement.

### 3. Render operations
Backend-neutral drawing instructions.

Keeping these separate is critical for testability and backend isolation.

---

## Traversal requirements

The scene tree must be easy to traverse for:
- focus collection,
- layout passes,
- style inheritance,
- diffing,
- collaboration overlays,
- diagnostics.

Karpal folds and traversals may be a useful long-term fit here.

---

## First-pass enum sketch

A rough direction might look like:

```rust
enum Scene<Msg> {
    Empty,
    Text(TextNode<Msg>),
    Row(ContainerNode<Msg>),
    Column(ContainerNode<Msg>),
    Stack(ContainerNode<Msg>),
    Grid(GridNode<Msg>),
    Dock(DockNode<Msg>),
    Sized(SizedNode<Msg>),
    Padding(PaddingNode<Msg>),
    Border(BorderNode<Msg>),
    Viewport(ViewportNode<Msg>),
    Scroll(ScrollNode<Msg>),
    Input(InputNode<Msg>),
    Editor(EditorNode<Msg>),
    List(ListNode<Msg>),
    Table(TableNode<Msg>),
    Canvas(CanvasNode<Msg>),
    Overlay(OverlayNode<Msg>),
    Annotated(AnnotatedNode<Msg>),
    Dynamic(DynamicNode<Msg>),
    ParticipantLayer(ParticipantLayerNode<Msg>),
}
```

This should not be treated as final, but it reflects the intended richness.

---

## Focus implications

The scene algebra must support focus from the start.

Likely scene-level focus features:
- focusable flag or focus node kind,
- focus order hints,
- nested focus scopes,
- selected/focused visual variants,
- explicit current-target identity in layouted scene or runtime metadata.

A later focus document should define exact semantics.

---

## Diff implications

Scene structure should be designed for efficient diffing.

Helpful properties:
- stable identity,
- normalized style/layout metadata,
- explicit content runs,
- deterministic child ordering,
- clear separation between semantic nodes and renderer ops.

Diffing should operate primarily on layouted scenes or normalized projections, not raw terminal output.

---

## First implementation recommendation

Start with a narrowed but structurally honest subset:
- `Text`
- `Row`
- `Column`
- `Stack`
- `Sized`
- `Padding`
- `Border`
- `Viewport`
- `Input`
- `List`
- `Annotated`

But preserve the broader algebra design in type organization and metadata.

---

## Open follow-up questions

1. exact style inheritance and override semantics
2. whether `Dynamic` exists as an explicit node or only via reactive projection
3. exact message binding representation on nodes
4. exact shape of collaboration annotations
5. whether tables/canvas are v0 or early-v0.1 additions

These should be refined in follow-up ADRs.
