# Knopper participant-local presence overlays

## Purpose

This document describes how Knopper should think about remote presence, remote cursors, remote selections, and participant attention indicators before a full collaborative runtime is implemented.

The goal is to ensure that future collaboration features layer cleanly onto the current machine/scene/runtime model without collapsing participant-local interaction into a fake global focus model.

## Core principle

Presence overlays should be treated as **participant-local derived presentation state**.

That means:

- they are rendered locally by each participant runtime
- they are derived from shared state and participant-published state
- they do not replace local focus or local cursor semantics
- they are overlays on semantic UI structure, not shared terminal-stream artifacts

In short:

> remote presence should be visible to a participant, but it should not hijack that participant’s local interaction model.

## Why this matters

A collaborative TUI must distinguish between:

- **my local focus and cursor**
- **shared semantic application state**
- **what other participants are doing**

If those are collapsed together, the UX becomes brittle and the architecture drifts toward single-global-user assumptions.

For Knopper, the correct stance is:

- local focus remains participant-local
- remote activity is rendered as semantically anchored overlays
- shared state remains the authoritative semantic substrate

## State model for presence

Presence overlays usually combine three kinds of state.

### 1. Shared replicated semantic state

Examples:

- shared document contents
- shared workspace tree
- shared list items
- shared task board state
- shared annotations

This is the semantic world participants are collaborating on.

### 2. Participant-local published state

Examples:

- participant cursor position within a document
- participant current selection range
- participant current viewport anchor
- participant active tool or mode
- participant current inspected/selected entity
- participant availability or role status

This state is often published to others, but it is not the same thing as global shared truth.

### 3. Local derived overlay state

Examples:

- “Alice is here” indicator near a node
- remote cursor glyphs
- remote selection highlights
- remote attention markers on list rows or panels
- participant badges or presence bars
- sync/conflict badges joined with participant context

This is what Knopper should actually render.

## Presence should not replace focus

This is the most important rule.

Remote presence should **not**:

- overwrite local `FocusState`
- force local tab traversal changes directly
- claim there is one globally authoritative focus path
- cause remote participants to steal keyboard ownership from the local participant

Instead:

- local focus remains local
- remote focus-like information is rendered as presence overlays
- shared semantic edits may still affect the UI, but not by pretending another participant became the local focus owner

## Semantic anchors for overlays

Presence overlays depend on stable anchors.

Knopper already has several promising anchoring mechanisms:

- `NodeId`
- scene structure
- annotations
- stable machine composition boundaries

### Preferred anchor types

Use semantically meaningful anchors such as:

- document region IDs
- list item IDs
- panel or workspace surface IDs
- editor node IDs
- machine root IDs
- selection/cursor anchors derived from shared document structure

### Avoid anchoring overlays to

- raw terminal coordinates as primary truth
- backend plane identities
- transient render-op ordering
- unstable generated IDs that change across reprojection

Coordinates may still be used as a **derived rendering result**, but not as the semantic source of truth.

## Scene-level representation options

There are several viable ways Knopper can express presence overlays in the future.

## Option A: overlay scenes

Presence can be projected as ordinary scene nodes layered structurally with `Scene::stack(...)` / `Scene::overlay(...)`.

Examples:

- a small badge over a panel corner
- a remote cursor indicator inside an editor surface
- a highlight layer over list items

### Pros

- works with the existing scene model
- compositional and structural
- easy to test through projection/layout/render

### Cons

- may become verbose for rich presence systems
- complex overlays may need helper abstractions

## Option B: annotations consumed by later passes

Presence intent can be attached as annotations and consumed by later layout/render passes.

Relevant existing annotation directions already include:

- `PresenceSlot(String)`
- `RemoteCursor(NodeId)`

### Pros

- keeps semantic intent explicit
- can support specialized render treatment later
- good for overlays that are conceptually attached to existing nodes

### Cons

- requires later passes to understand more semantics
- can become opaque if overused

## Option C: hybrid approach

This is likely the best long-term direction.

Use:

- scene overlays for visible semantic layering
- annotations for attachment points and specialized overlay semantics

That gives Knopper both clarity and flexibility.

## Recommended first-pass approach

For the near term, use a hybrid rule of thumb:

1. anchor presence to stable semantic `NodeId`s and/or shared document anchors
2. express visible presence with ordinary scene overlays where possible
3. use annotations when presence needs a semantic attachment point or future-specialized rendering

## Common presence overlay patterns

## Remote cursor

Examples:

- another participant’s editor caret
- a participant pointing at a list item
- a participant’s current command target

Recommended semantics:

- participant-local published state identifies the semantic anchor
- local projection renders a cursor/marker overlay attached to that anchor
- local focus remains independent

## Remote selection highlight

Examples:

- another participant selecting text
- selecting a list range
- selecting a tree node or task card

Recommended semantics:

- shared or participant-published selection anchors determine the semantic region
- local projection renders a highlight layer or border accent
- selection visibility may be personalized by theme/style per participant

## Attention marker

Examples:

- “Alice is inspecting this panel”
- “Agent B is working in this task column”
- “Reviewer is on this row”

Recommended semantics:

- render subtle structural overlays or annotations near the semantic surface
- do not treat this as local focus takeover

## Presence bar / participant strip

Examples:

- active participants in current workspace
- participants editing a specific surface
- sync badges plus participant state

Recommended semantics:

- derive from participant registry plus scoped semantic anchors
- render as dedicated scene nodes, likely outside the primary interactive focus graph

## Viewport and visibility semantics

Presence overlays interact with local viewport/render constraints.

Important rule:

> presence is semantically shared, but visibility is local.

Therefore:

- a remote cursor may exist on a semantic anchor outside the local viewport
- the local participant may render:
  - nothing,
  - an offscreen indicator,
  - a scrollbar marker,
  - a minimap or status hint,
  depending on the app

Knopper should not require all participants to share one viewport.

## Participant identity and styling

Presence overlays need participant identity data.

Typical participant metadata:

- participant ID
- display name
- color/style identity
- role/capability
- human/agent classification
- availability/status

Styling implications:

- participant colors can be rendered locally
- local themes may remap or soften remote participant styles
- presence should remain legible without forcing a global style system

## Interaction implications

Presence overlays may be visible, but should usually not become active focus participants unless explicitly designed to be interactive.

Default guidance:

- remote presence overlays should not be focusable by default
- they should not disrupt local tab order unless intentionally interactive
- if interactive overlays are needed, they should opt in explicitly via scene semantics

## Machine implications

Machines should remain responsible for projecting presence-relevant overlays when they have the necessary local/shared inputs.

This suggests a future shape such as:

- local model for participant-local interaction state
- shared state for semantic shared data
- participant registry / published presence inputs
- derived scene combining all three

The important point is that presence belongs in projection, not as an afterthought bolted onto backend rendering.

## Runtime implications

A future collaboration-capable runtime may eventually ingest:

- local input events
- shared state updates
- participant presence updates

But the runtime should still preserve the distinction between:

- local focus and keyboard handling
- remote presence rendering
- shared semantic state propagation

The current local runtime is still compatible with this direction.

## Standard-machine implications

## Input and editor-like surfaces

Presence overlays here may include:

- remote cursor
- remote selection
- remote edit ownership hint

Guidance:

- keep current `InputMachine` framed as local-first
- treat richer collaborative text editing as a future abstraction

## Lists and trees

Presence overlays here may include:

- remote selected row marker
- participant attention badge on items
- shared status markers joined with participant identity

Guidance:

- use stable item IDs
- keep local navigation/focus local
- render remote attention as decoration, not focus takeover

## Modals and command surfaces

Presence overlays here may include:

- participant using a shared command surface
- reviewer attention in a modal workspace

Guidance:

- local modal focus remains participant-local
- remote modal activity should appear as metadata/presence, not as local focus replacement

## Minimum design commitments before full implementation

Before Knopper implements a full presence system, it should maintain these commitments.

### Must preserve

- stable semantic anchors
- participant-local focus semantics
- scene-based projection as the primary integration point
- backend-local rendering and caching
- room for annotations and overlays to coexist

### Should avoid

- globally shared focus assumptions
- renderer-level presence hacks with no semantic anchor
- APIs that assume one cursor or one selection is authoritative for all participants

## Recommended next implementation-adjacent steps

1. keep stable `NodeId` practices central in new standard machines
2. when adding richer controls, note likely presence anchor points in docs/tests
3. consider small helper conventions for participant-themed overlay styling
4. later, add a tiny example of a locally rendered remote attention marker using existing scene/annotation primitives

## Downstream contract

Downstream IA projects should be able to assume:

- Knopper intends remote presence to be rendered as semantic overlays
- local focus remains local to a participant runtime
- collaborative visibility is derived from shared plus participant-published state
- future collaboration features should deepen the current architecture rather than replace it

That is the intended meaning of participant-local presence overlays in Knopper.
