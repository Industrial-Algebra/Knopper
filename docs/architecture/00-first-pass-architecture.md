# Knopper First-Pass Architecture

## Status

Draft architecture specification.

## Thesis

Knopper is a collaborative functional-reactive TUI framework and terminal rendering engine in Rust.

It is built around **Machines** as the primary public abstraction. Machines project reactive terminal scenes through lawful transformation pipelines into a Notcurses backend.

Knopper is intentionally:

- **not** React-for-the-terminal
- **not** DOM-shaped
- **not** JSX-first
- **not** a direct wrapper around Notcurses primitives
- **not** limited to single-user local interaction

Knopper is instead:

- a **terminal-native scene algebra**
- a **functional-reactive projection system**
- a **hybrid local-state architecture** combining messages and FRP
- a **collaborative runtime** capable of semantic multi-user synchronization
- a **lawful algebraic framework** using optics, monoids, prisms, and compositional pipelines where practical

---

## Architectural principles

1. **Machines, not components**
   - The public abstraction is the Machine.
   - A Machine is an algebraic projection from inputs, context, and state into a reactive scene.

2. **Scenes, not DOM**
   - Knopper uses a terminal-native scene algebra instead of HTML-like nodes.

3. **Reactive projection first**
   - The semantic output of a Machine is `Behavior<Scene<Msg>>`.

4. **Hybrid state model**
   - Message-driven updates and FRP-style derived state coexist.

5. **Semantic collaboration**
   - Synchronization occurs at the level of shared application state and scene semantics, not terminal byte streams.

6. **Renderer isolation**
   - Notcurses is a backend implementation detail behind renderer abstractions.

7. **Lawful composition**
   - Style composition, patch accumulation, routing, optics, and layout transforms should follow algebraic laws where practical.

8. **Identity-rich scene graph**
   - Stable node identity is a core design requirement for diffing, focus management, event routing, and collaboration overlays.

---

## Ecosystem dependencies and intended roles

### Cliffy

Knopper should leverage Cliffy for functional reactive state and collaborative synchronization.

#### `cliffy-core`
Use for:
- `Behavior<T>`
- `Event<T>`
- reactive combinators
- time-varying scene projection
- derived layout and style computations
- geometric state where it improves layout, animation, interpolation, or spatial reasoning

#### `cliffy-protocols`
Use for:
- semantic multi-user synchronization
- geometric CRDT-backed shared state
- causal ordering / vector clocks
- distributed presence and convergence
- collaborative application state for multi-user TUIs and agent environments

### Orlando

Knopper should leverage Orlando's transducer-oriented architecture for runtime transformation pipelines.

Use for:
- input normalization pipelines
- event routing pipelines
- scene transformation and annotation passes
- layout preparation and projection passes
- render patch generation
- diff fusion and bounded-memory processing

### Karpal

Knopper should leverage Karpal for algebraic correctness and composition.

Use for:
- semigroup/monoid-based style composition
- optics for nested machine state
- prisms for message routing and sum-type decomposition
- traversals/folds over scene trees
- lawful composition of update and projection machinery

---

## High-level architecture

Knopper should be organized around five layers.

### 1. Reactive domain layer
Responsible for:
- local machine state
- shared replicated state
- events and behaviors
- derived reactive values
- subscriptions and temporal signals

Primary tools:
- `cliffy-core`
- `cliffy-protocols`
- Karpal optics where useful

### 2. Machine algebra layer
Responsible for:
- user-facing Machine abstraction
- composition of child machines
- typed messages and routing
- local/shared state boundaries
- projection contracts

Primary tools:
- Knopper core traits and types
- Karpal prisms/optics

### 3. Projection and transformation layer
Responsible for:
- converting machine state into scene algebra
- scene transformation passes
- annotation and enrichment
- collaboration overlays
- layout preparation

Primary tools:
- `Behavior<Scene<Msg>>`
- Orlando-style transducer pipelines

### 4. Rendering layer
Responsible for:
- scene diffing
- render op generation
- backend-specific resource management
- Notcurses lowering and commit

Primary tools:
- Knopper renderer abstractions
- Notcurses backend implementation
- Rayon for CPU-bound layout/diff work where valid

### 5. Runtime and event layer
Responsible for:
- terminal input capture
- event normalization
- focus routing
- async completions
- timer/tick handling
- collaboration events
- runtime loop orchestration

Primary tools:
- Knopper runtime
- Orlando-style event pipelines
- Cliffy events and behaviors

---

## Core dataflow model

The intended runtime shape is:

```text
Raw Terminal Input / Async Events / Sync Events
-> Normalize
-> Enrich with Runtime Context
-> Route by Focus / Address / Identity
-> Lift into Machine Messages
-> Update Local and Shared State
-> Recompute Derived Behaviors
-> Project Behavior<Scene<Msg>>
-> Run Scene Transformation / Collaboration Annotation Passes
-> Resolve Layout
-> Diff Against Previous Layouted Scene
-> Lower to Render Operations
-> Commit Through Renderer Backend
```

This should be understood as a semantic pipeline, not necessarily a single monolithic function.

---

## Machine model

### Core idea

A Machine is the primary public unit of composition.

A Machine consumes context and input, evolves local state, observes shared state, and projects a time-varying scene.

Conceptually:

```text
Machine<Ctx, Input, Msg> : Ctx × Event<Input> -> Behavior<Scene<Msg>>
```

### Design recommendation

Knopper should provide:
- a **core trait** used by the runtime
- ergonomic wrappers/builders/combinators for authoring and composition

### First-pass trait sketch

```rust
trait Machine {
    type Context;
    type Input;
    type Msg;
    type Model;
    type Shared;

    fn init(&self, ctx: &Self::Context) -> Self::Model;

    fn update(
        &self,
        model: &mut Self::Model,
        msg: Self::Msg,
        ctx: &Self::Context,
    ) -> Effects<Self::Msg>;

    fn project(
        &self,
        model: cliffy_core::Behavior<Self::Model>,
        shared: cliffy_core::Behavior<Self::Shared>,
        ctx: &Self::Context,
    ) -> cliffy_core::Behavior<Scene<Self::Msg>>;
}
```

This is a design sketch, not a committed API.

### Why this shape

This trait shape explicitly models:
- local machine state (`Model`)
- collaborative/replicated state (`Shared`)
- typed messages (`Msg`)
- reactive scene projection

It also leaves room for:
- runtime-managed subscriptions
- child machine embedding
- async effects / commands
- structured collaboration support

---

## Machine authoring model

Although the semantic output is `Behavior<Scene<Msg>>`, simple Machines should not require users to manually construct reactive graphs for trivial cases.

Knopper should support two authoring modes:

### 1. Static/pure scene projection
For simple Machines that can project directly from current state.

Conceptually:

```rust
fn view(&self, model: &Model) -> Scene<Msg>
```

The runtime can lift this into a constant or derived `Behavior<Scene<Msg>>`.

### 2. Fully reactive projection
For Machines that need direct reactive composition.

Conceptually:

```rust
fn project(
    &self,
    model: Behavior<Model>,
    shared: Behavior<Shared>,
    ctx: &Context,
) -> Behavior<Scene<Msg>>
```

This dual approach preserves the reactive semantic core without making authoring ergonomics unnecessarily heavy.

---

## State model

Knopper should explicitly support three state domains.

### 1. Local ephemeral state
Not replicated by default.

Examples:
- focus state
- cursor blink / animation state
- viewport caches
- transient input capture
- local measurement caches
- local-only overlays

### 2. Shared replicated state
Replicated and convergent via `cliffy-protocols`.

Examples:
- collaborative document contents
- shared pane layouts
- shared selections and annotations
- multi-user command/task state
- collaborative AI-agent workspace state
- shared logs or semantic outputs

### 3. Derived reactive state
Computed from local + shared state.

Examples:
- collaboration overlays
- presence summaries
- sync status indicators
- effective focus decorations
- derived scene slices
- layout dependencies

This separation is essential because not all TUI state should be synchronized, but collaboration should be a first-class capability.

---

## Message and event model

Knopper should distinguish several layers of events.

### Raw runtime inputs
Examples:
- key input
- mouse input
- paste events
- terminal resize
- tick/timer events
- renderer/backend notifications
- async completion events
- synchronization / remote update events

### Routed scene/runtime events
Examples:
- focus gained/lost
- activation
- selection changed
- scroll intent
- command invocation
- input edit intent
- remote presence update

### Machine messages
User-defined typed intents that drive local updates and shared actions.

### Recommendation

Scene nodes should not rely primarily on arbitrary closures for behavior.
Instead, they should carry typed semantic intents/messages or references to structured runtime routing.

This better supports:
- serialization or introspection where needed
- collaborative routing
- deterministic testing
- child machine composition
- diff stability

---

## Scene algebra

Knopper should start with a richer scene algebra rather than a minimal one, because future concerns such as focus, scrolling, overlays, and collaboration affect foundational design.

### Requirements

A `Scene<Msg>` should:
- be terminal-native rather than DOM-like
- support stable identity
- support styling and inherited style composition
- support layout structure
- support interactivity
- support overlays and z-order
- support collaboration-aware annotations
- be traversable and diffable

### First-pass scene categories

#### Structural/layout nodes
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

#### Content nodes
- `Text`
- `Span`
- `Paragraph`
- `List`
- `Table`
- `Canvas`
- `Rule`
- `Border`
- `Surface`

#### Interactive nodes
- `Focusable`
- `Input`
- `Editor`
- `Selectable`
- `CommandSurface`
- `Shortcut`
- `Action`

#### Reactive/control nodes
- `Conditional`
- `Switch`
- `Dynamic`
- `Portal`
- `Annotated`

#### Collaboration-aware nodes
- `Presence`
- `RemoteCursor`
- `RemoteSelection`
- `SharedViewport`
- `ConflictOverlay`
- `SyncStatus`
- `ParticipantLayer`

These are category targets for the initial algebra, not a finalized exhaustive enum.

---

## Scene identity and metadata

Stable identity is required from the start.

Each scene node should be able to carry at least:
- `NodeId`
- role or semantic kind
- optional address/path metadata
- arbitrary annotations/tags

### Why identity matters

Stable identity enables:
- efficient diffing
- focus restoration across reprojection
- deterministic event routing
- collaboration overlays anchored to semantic nodes
- remote cursor/selection attachment
- renderer resource reuse

Without stable identity, collaborative features and intelligent patching become significantly harder.

---

## Focus and routing model

Focus should be a first-class core concern.

A TUI framework lives or dies on focus semantics, keyboard routing, selection behavior, and command targeting.

### First-pass focus requirements

Knopper should support:
- focusable scene nodes
- explicit focus order / traversal
- semantic focus identity
- nested focus scopes
- local focus overlays
- remote focus/presence decorations where collaboration requires them

### Event routing requirements

Routing should consider:
- current focus target
- node identity/path
- machine boundaries
- message prisms for child composition
- collaboration-aware event addressing where necessary

---

## Child machine composition

Child Machines should compose through:
- scene embedding
- optics over child/local/shared state
- prisms over parent/child messages

### Recommendation

Use Karpal optics and prisms to model composition boundaries.

This allows a parent Machine to:
- focus into child model slices
- route child messages into parent message sums
- preserve lawful, structured composition

This should be preferred over ad hoc callback plumbing.

---

## Projection and transformation pipeline

Knopper should make its transformation pipeline explicit.

### Recommended projection stages

1. **Machine projection**
   - produce `Behavior<Scene<Msg>>`
2. **Scene annotation**
   - enrich with focus, theme, presence, sync state
3. **Layout resolution**
   - compute bounds, clipping, z-order, alignment, sizing
4. **Collaboration overlays**
   - remote cursors, selections, participant layers, status markers
5. **Diff preparation**
   - compare with previous layouted scene
6. **Render op generation**
   - generate backend-neutral operations
7. **Backend lowering**
   - translate render ops to Notcurses primitives
8. **Commit**
   - perform renderer commit

### Orlando role

These stages are natural candidates for Orlando-inspired transducer pipelines.

The pipeline architecture should aim for:
- composability
- bounded intermediate allocations
- pass fusion where possible
- testable intermediate representations

---

## Rendering architecture

Notcurses should be treated as a backend, not as the public model.

### Backend responsibilities
- plane allocation and reuse
- cell/grapheme rendering
- style application
- clipping and z-order handling
- cursor management
- advanced terminal effects supported by Notcurses
- atomic or coherent render commits

### Intermediate render representations

Knopper should maintain at least three render-related representations:

#### 1. Declarative scene
User- and Machine-facing structure.

#### 2. Layouted scene
Resolved geometry, bounds, effective styles, clipping, and ordering.

#### 3. Render operations
Backend-neutral instructions such as:
- draw text run
- set style region
- draw border
- clear region
- move cursor
- create/update/destroy surface or plane

This separation preserves testability and backend decoupling.

---

## Concurrency model

Knopper should use `rayon` where CPU-bound parallelism is beneficial, but it should assume final backend mutation may need serialization.

### Likely good parallel candidates
- layout computation over independent subtrees
- text measurement/cache preparation
- diff preparation
- render op batching
- collaboration annotation passes over large scenes

### Likely serialized candidates
- final Notcurses plane mutation
- terminal I/O commit
- some runtime state coordination steps

This should be validated against Notcurses constraints rather than assumed.

---

## Collaboration model

Collaboration is a first-class architectural pillar.

Knopper should model collaborative TUIs semantically, not as shared terminal streams.

### Core principle

Multiple users participate in a shared reactive state space.
Each participant renders a local terminal projection of convergent shared state plus local ephemeral state.

### This enables
- per-user theming or viewport adaptation
- local ephemeral focus and overlays
- convergent shared data
- remote presence markers
- collaborative editing and navigation
- shared agent dashboards and coding workspaces

### Use cases
- collaborative coding-agent control rooms
- multi-user debugging consoles
- shared terminal workspaces
- synchronized operator dashboards
- collaborative task orchestration interfaces
- semantic pair-programming TUIs

### Collaboration layers

Knopper should eventually support:
- shared replicated state via `cliffy-protocols`
- participant identity and presence
- remote cursor and selection overlays
- synchronization status and conflict visualization
- semantic, addressable scene anchors

---

## AI agent suite relevance

Knopper's collaborative architecture makes it particularly well suited as a foundation for collaborative AI coding agents and operator-facing terminal systems.

Potential application domains include:
- shared coding environments
- agent swarm orchestration
- collaborative code review TUIs
- synchronized command pipelines
- multi-user planning/task decomposition dashboards
- shared observability and trace consoles

This is a strategic differentiator and should be treated as a primary design target rather than an incidental feature.

---

## Proposed crate/module structure

A likely first-pass workspace shape:

```text
knopper/
├── knopper-core        # Machine traits, scene algebra, style algebra, IDs, events
├── knopper-runtime     # Runtime loop, routing, subscriptions, projection pipeline
├── knopper-layout      # Layout engine and layouted scene representation
├── knopper-render      # Backend-neutral render ops, diffing, renderer traits
├── knopper-notcurses   # Notcurses backend implementation
├── knopper-sync        # Collaborative integration layer over cliffy-protocols
├── knopper-widgets     # Standard Machines and higher-level scene builders
└── knopper-macros      # Optional future syntax/ergonomic macros
```

This can begin as modules within one crate and split later, or begin as a workspace once the boundaries are proven.

### If starting in a single crate
A likely internal module layout:

```text
src/
├── machine/
├── scene/
├── style/
├── event/
├── focus/
├── runtime/
├── layout/
├── render/
├── backend/
└── sync/
```

---

## Initial milestone recommendation

Before advanced collaboration features or large widget catalogs, Knopper should validate the architecture with a small but representative vertical slice.

### Suggested initial scope
- Machine trait and basic runtime wiring
- identity-rich scene algebra
- basic style algebra
- focus model foundations
- layout for row/column/stack/sized/padding/text
- backend-neutral render ops
- Notcurses renderer backend
- reactive counter/list/input demo
- basic collaboration-aware node placeholders even if not fully implemented

### Demo goals
- local reactive state updates
- focusable elements
- text input
- split-pane layout
- resize handling
- diffed redraw
- basic shared-state proof-of-concept later in the milestone sequence

---

## Open questions for later ADRs

These items should be resolved in follow-up architecture records:

1. exact `Machine` trait API and effect/subscription model
2. concrete `Scene<Msg>` enum and node builder API
3. style algebra and inheritance semantics
4. focus traversal and nested scope semantics
5. exact renderer trait boundaries
6. exact `cliffy-protocols` integration surface
7. whether collaboration is always available or feature-gated
8. whether scene handlers are represented as typed intents, command descriptors, or runtime addresses
9. how much of Orlando is depended on directly vs reinterpreted architecturally in Knopper
10. crate split timing versus single-crate incubation

---

## Immediate next documents recommended

To refine this first-pass architecture, Knopper should next define:

1. `docs/architecture/01-machine-model.md`
2. `docs/architecture/02-scene-algebra.md`
3. `docs/architecture/03-collaboration-model.md`
4. `docs/architecture/04-runtime-pipeline.md`
5. `docs/architecture/05-rendering-model.md`

These should become the basis for implementation-focused ADRs and TDD scaffolding.
