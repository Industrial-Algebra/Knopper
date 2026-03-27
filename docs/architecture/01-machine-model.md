# Knopper Architecture: Machine Model

## Status

Draft.

## Purpose

This document refines the Knopper **Machine** abstraction, the primary public unit of composition in the framework.

A Machine is not a React-style component and not a direct wrapper around a terminal surface. A Machine is an algebraic projection that:

- owns or observes state,
- receives typed input and messages,
- may produce effects,
- and projects a reactive terminal scene.

---

## Core definition

Conceptually:

```text
Machine<Ctx, Input, Msg> : Ctx × Event<Input> -> Behavior<Scene<Msg>>
```

More operationally, a Machine:

1. initializes local state,
2. consumes routed messages,
3. observes local and shared state,
4. projects `Behavior<Scene<Msg>>`,
5. composes with child Machines.

---

## Design goals

The Machine model should provide:

- **terminal-native semantics**
- **reactive projection**
- **hybrid state management**
- **typed message routing**
- **composable child embedding**
- **collaboration-aware state boundaries**
- **testability without a renderer**

---

## Core associated roles

A first-pass Machine should model these roles explicitly:

- `Context`: runtime and application context
- `Input`: routed external/runtime input
- `Msg`: typed machine intent
- `Model`: local machine state
- `Shared`: replicated/shared collaborative state
- `Effects`: commands, subscriptions, or emitted work

---

## First-pass trait sketch

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

This is still a design sketch.

---

## Why a trait plus ergonomic wrappers

Knopper should expose:

1. a **core trait** for runtime integration and principled composition,
2. **ergonomic builders/wrappers** for common authoring styles.

### Why not only a trait?
Because many Machines will want:
- simple static views,
- common update loops,
- child composition helpers,
- easier effect wiring.

### Why not only a struct API?
Because Knopper needs a stable semantic core for:
- runtime execution,
- embedding,
- testing,
- law-driven composition.

---

## Two authoring modes

### 1. Pure projection authoring
For simple Machines, the author can think in terms of:

```rust
fn view(&self, model: &Model, shared: &Shared, ctx: &Context) -> Scene<Msg>
```

The runtime lifts this into a `Behavior<Scene<Msg>>`.

This should be the common ergonomic path.

### 2. Reactive projection authoring
For more advanced Machines, the author may directly construct:

```rust
fn project(
    &self,
    model: Behavior<Model>,
    shared: Behavior<Shared>,
    ctx: &Context,
) -> Behavior<Scene<Msg>>
```

This mode is useful when:
- the scene depends on derived reactive values,
- animation or interpolation matters,
- collaborative overlays are dynamic,
- scene subtrees are naturally behavior-valued.

---

## Hybrid state model inside a Machine

Knopper should not force a purely TEA-like or purely FRP-like style.

A Machine should support both:

### Message/update state
Useful for:
- user intents,
- domain transitions,
- effect boundaries,
- async completion,
- deterministic reducer-like logic.

### FRP-derived state
Useful for:
- derived layout,
- time-varying visual state,
- animation,
- collaboration overlays,
- focus-dependent projections,
- shared/local state fusion.

### Recommended interpretation
A Machine uses messages to change local state, but projection is fundamentally reactive.

---

## Local vs shared state

A Machine should clearly separate:

### Local `Model`
Machine-owned ephemeral or app-local state.

Examples:
- local input buffer,
- selection anchor,
- local viewport,
- widget expansion state.

### Shared `Shared`
Replicated state with semantic convergence.

Examples:
- collaborative document,
- shared workspace layout,
- synchronized task board,
- remote participant metadata.

This separation should remain explicit in the API.

---

## Machine context

`Context` should provide runtime and application services without coupling the Machine to Notcurses.

Likely responsibilities:
- terminal capabilities,
- theme access,
- clock/timer services,
- environment/config,
- collaboration/session metadata,
- effect dispatch handles,
- renderer-independent layout metrics.

Context should be an abstraction boundary, not a bag of global mutable state.

---

## Effects model

`update` should be able to return a structured effect value.

A first-pass effect algebra may include:
- emit another message,
- schedule async work,
- subscribe to an event source,
- write to shared state,
- request focus,
- request tick/timer,
- no-op.

Conceptually:

```rust
enum Effect<Msg> {
    None,
    Emit(Msg),
    Batch(Vec<Effect<Msg>>),
    RequestFocus(NodeId),
    Spawn(TaskSpec<Msg>),
    Subscribe(SubscriptionSpec<Msg>),
    Shared(SharedEffect),
}
```

This should later become an ADR.

---

## Child Machine composition

A parent Machine should compose child Machines through:

- **state optics** into child model/shared slices,
- **message prisms** into parent/child sum types,
- **scene embedding** into the parent scene.

This should be the preferred composition pattern.

### Why optics and prisms?
They make machine composition:
- typed,
- explicit,
- lawful,
- easier to test,
- less callback-heavy.

Karpal should be the primary supporting library here.

---

## Message routing model

Messages should be typed machine intents, not arbitrary closures captured deep in scene nodes.

This improves:
- event routing,
- composition,
- serialization or introspection where needed,
- deterministic testing,
- collaboration-aware replay.

Parent Machines should define sum-type messages and use prisms to route child messages.

---

## Machine lifecycle

A first-pass Machine lifecycle likely includes:

1. `init`
2. `mount` or runtime registration
3. `update` on routed messages
4. `project` continuously / reactively
5. `teardown` or runtime drop

A separate explicit lifecycle API may or may not be needed initially, but the runtime should account for:
- subscriptions,
- async tasks,
- child machine resource cleanup,
- shared-state detachment.

---

## Identity and addressability

Machines should be addressable within the runtime.

Important identities include:
- machine instance identity,
- scene node identity,
- focus path,
- collaboration anchor IDs.

This is necessary for:
- focus restoration,
- child routing,
- multi-user overlay attachment,
- async/event reply targeting.

---

## Testing implications

A Machine should be testable in isolation without Notcurses.

Useful test surfaces:
- `init` state correctness,
- `update` transitions,
- effect emission,
- `view`/`project` output,
- child routing behavior,
- collaboration behavior given shared state changes.

This suggests Knopper should provide testing helpers for:
- fake contexts,
- fake input streams,
- behavior sampling,
- scene assertions.

---

## First implementation recommendation

The first implementation should support:
- a minimal `Machine` trait,
- a simple pure-view wrapper,
- a basic effect type,
- child composition via state/message adapters,
- tests proving parent/child composition.

Do not begin with advanced proc macros. Stabilize the semantic model first.

---

## Open follow-up questions

1. exact effect algebra
2. sync between `update` and reactive behavior graph maintenance
3. explicit lifecycle hooks vs runtime-managed drop semantics
4. machine instance IDs and addressing scheme
5. whether `Input` remains a direct associated type or is always normalized into `Msg`

These should be resolved in later ADRs.
