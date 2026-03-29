# Knopper collaboration-readiness guide

## Purpose

This document defines the collaboration-readiness contract for Knopper prior to a full collaborative runtime.

It exists to ensure that ongoing work on focus, runtime policy, standard machines, layout, and rendering does not accidentally hardcode single-user assumptions into APIs that downstream IA projects will depend on.

This is not yet a full protocol or distributed-runtime spec. It is a design guardrail document for getting Knopper to a useful first release while preserving its core differentiator: semantic collaborative TUI architecture.

## Core stance

Knopper should be useful as a single-user framework in `0.1.0`, but it must be designed as a collaboration-capable framework from the start.

That means:

- single-user behavior may be the initial default
- collaboration does not need to be fully implemented in `0.1.0`
- but all major APIs should preserve the ability to layer in:
  - replicated shared state
  - participant-local state
  - presence overlays
  - remote cursors and selections
  - convergence-aware update flows

## Collaboration-ready invariants

The following invariants should guide all near-term implementation work.

### 1. Preserve state separation

Knopper already has the right high-level split in the `Machine` trait:

- `Model`
- `Shared`
- `project(model, shared, ctx) -> Behavior<Scene<Msg>>`

For collaboration readiness, this split must remain semantically meaningful.

#### `Model`
Represents local machine state.

Typical examples:

- local focus path
- local viewport position when purely participant-specific
- local editor cursor if not intentionally shared
- transient input editing mechanics
- IME or composition state
- local animation and blink state

#### `Shared`
Represents replicated or externally synchronized semantic state.

Typical examples:

- document contents
- shared task state
- shared command history
- replicated annotations
- participant presence registry
- shared selection model when the application intends that to converge

#### Derived projection
The projected scene should remain a pure function of:

- local model
- shared state
- context

This is the seam where collaborative overlays will eventually appear.

### 2. Do not collapse local and shared concerns for convenience

A machine should not store shared semantic truth in `Model` just because a single-user demo is easier that way.

If a piece of state is expected to become replicated in downstream applications, prefer one of the following:

- keep it in `Shared`
- document why it is currently local-only
- design the machine so a future shared-backed variant does not require a conceptual rewrite

### 3. Treat focus as participant-local by default

Current Knopper focus behavior should be understood as local-to-one-runtime, not globally authoritative.

For future collaboration:

- each participant may have their own focus path
- local focus traversal should not imply globally shared focus ownership
- remote focus may be rendered as presence or annotation overlays rather than replacing local focus

Therefore, current focus APIs should avoid assumptions like:

- there is exactly one globally meaningful focus state for an application
- focus changes inherently mutate shared application state

The current `Runtime`-owned `FocusState` is acceptable for now as a local participant runtime concern.

### 4. Distinguish local editing mechanics from shared semantic edits

Some controls, especially text-oriented ones, need careful separation.

For example, an input or editor may involve:

- local cursor position
- local in-progress composition state
- shared document content
- remote selections/cursors

For `0.1.0`, a simple local input control is acceptable, but its public API should not imply that all editing state must always be local and isolated.

### 5. Keep scene identity stable and semantic

Collaboration overlays depend on stable anchors.

That means `NodeId` usage should continue to favor semantic stability over incidental render-time convenience.

Stable scene identities are important for:

- remote cursor anchoring
- participant presence indicators
- shared selection ranges
- conflict indicators
- attachment of replicated annotations to semantic surfaces

### 6. Keep runtime and renderer state unshared by default

The renderer/backend layer should remain local.

Do not design collaboration around sharing:

- terminal byte streams
- renderer surface caches
- Notcurses plane state
- local layout caches
- terminal dimensions

Collaboration should synchronize semantic state, not terminal implementation details.

## State taxonomy for implementation work

When introducing a new machine or runtime feature, classify its state into one of four buckets.

### A. Local ephemeral state

Not replicated by default.

Examples:

- local focus
- local cursor blink
- local scroll offset when viewport is participant-specific
- local measurement caches
- local theme
- temporary drag/selection mechanics

### B. Shared replicated state

Semantically convergent and intended to sync.

Examples:

- document data
- shared command registry state
- synchronized task graph
- shared annotations
- participant roster
- remote-presence metadata

### C. Participant-local published state

Participant-owned state that may be published to others, but is not the same thing as global shared truth.

Examples:

- remote cursor positions
- remote selection highlights
- participant viewport anchors
- presence status
- active participant tool/mode

This category is especially important because it often gets confused with either purely local state or globally shared state.

### D. Derived presentation state

Computed from local, shared, and participant-local inputs.

Examples:

- remote cursor overlays
- presence bars
- divergence markers
- sync badges
- “Alice is editing here” indicators

## Implications for current core APIs

## `Machine`

The current trait is directionally correct:

- `Model` already gives a local state lane
- `Shared` already gives a replicated/shared lane
- projection already combines them

Near-term guidance:

- preserve this split in all standard machines
- avoid adding convenience APIs that implicitly erase the distinction
- when a machine only uses `Shared = ()` today, document whether that is fundamentally local or merely a first-pass simplification

## `Runtime`

Current `Runtime` is a local runtime for one participant session.

That is acceptable for now.

Near-term guidance:

- treat `focus` as local runtime state
- treat renderer/backend state as local runtime state
- keep `set_shared(...)` as an important seam for externally synchronized updates
- avoid assuming that all meaningful state transitions originate from a single local input stream forever

Longer-term direction:

- a collaboration-capable runtime may ingest remote shared-state updates and participant presence streams alongside local events

## `Scene`

The scene model should continue to represent semantic UI structure, not terminal implementation details.

Near-term guidance:

- focus scopes should be structural and semantic
- annotations should remain viable anchors for future collaboration overlays
- do not overload scene wrappers with assumptions that only make sense for single-user rendering

## `NodeId`

Node identity should remain stable where possible across reprojection.

Near-term guidance:

- prefer semantically derived IDs for persistent surfaces
- avoid unstable IDs when they would break overlay anchoring or patch coherence

## Standard-machine guidance

## `ListMachine`

Current state:

- `selected`
- `scroll`

Collaboration guidance:

- `scroll` is usually participant-local
- `selected` may be local or shared depending on application semantics
- therefore `ListMachine` should not be documented as inherently owning globally shared selection semantics

Downstream expectation:

- it should remain possible to wrap or adapt the machine so selection comes from shared state when desired

## `InputMachine`

Current state:

- `value`
- `cursor`
- `committed`

Collaboration guidance:

- this is acceptable as a local-first control for `0.1.0`
- but it should be described as a local input field, not as the final abstraction for collaborative text editing
- future collaborative editors will likely separate:
  - shared text content
  - participant-local cursor/composition state
  - published remote cursor/selection state

## Modal and command palette behavior

Collaboration guidance:

- modal open/closed state may be local or shared depending on application semantics
- focus within modals should remain participant-local by default
- remote participants should not implicitly steal local modal focus

## Questions to ask for every new machine

Before merging a new standard machine, answer these questions.

1. Which parts of this machine’s state are strictly local?
2. Which parts might be shared in a collaborative application?
3. Which parts are participant-local but publishable to others?
4. Does the public API accidentally assume a single global focus or selection owner?
5. Could a downstream project adapt this machine to shared state without changing the core abstraction?
6. Are node identities stable enough for overlays and presence indicators?

## Questions to ask for every runtime feature

1. Is this runtime state local, shared, participant-local, or derived?
2. Does this behavior assume a single participant is authoritative?
3. If a remote update arrives, is there a clean seam for integrating it?
4. Does the behavior belong in semantic state or only in local rendering/runtime state?
5. Will this make future collaboration overlays harder to express?

## Minimum collaboration-readiness bar for `0.1.0`

Knopper `0.1.0` should satisfy the following minimum bar.

### Must have

- explicit documentation of local vs shared state lanes
- a runtime model that can accept external shared-state updates
- standard machines that do not unnecessarily hardcode single-user semantics into public contracts
- focus treated as local participant state
- stable semantic scene/node identity conventions

### Should have

- examples or notes showing how shared state would feed projection
- at least one documented downstream integration pattern for collaborative applications
- explicit notes on participant-local published state such as remote presence/cursors

### Not required yet

- full multi-user runtime implementation
- wire protocol integration
- CRDT-backed editors in-tree
- complete remote-presence rendering system

## Recommended near-term follow-up work

1. audit existing standard machines against this document
2. annotate roadmap tasks with collaboration-readiness checks
3. keep focus policy work explicitly participant-local by default
4. write one small design note on participant-local presence overlays
5. ensure the first polished demo does not imply that all state is single-user-owned

## Contract for downstream IA projects

Downstream projects should be able to rely on the following:

- Knopper’s core model is intentionally split between local model state and shared state
- current runtime focus behavior is participant-local, not a claim about global collaborative focus
- scene projection is the intended place to combine local, shared, and future collaboration-oriented overlays
- the first-release API is being shaped to avoid a later architectural inversion when collaborative features deepen

That is the key meaning of collaboration-readiness for the first release.
