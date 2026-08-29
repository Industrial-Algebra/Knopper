# The Machine Model

Everything in Knopper is a **machine**. A machine is the complete specification
of a UI unit: what state it holds, what messages it understands, and what scene
it projects.

```rust
pub trait Machine {
    type Context;  // app-level, relatively static (identity, theme, policy)
    type Msg;      // messages the machine understands
    type Model;    // participant-local UI state
    type Shared;   // host-pushed projection state

    fn init(&self, ctx: &Self::Context) -> Self::Model;

    fn update(
        &self,
        model: &mut Self::Model,
        msg: Self::Msg,
        ctx: &Self::Context,
    ) -> Effect<Self::Msg>;

    fn project(
        &self,
        model: Behavior<Self::Model>,
        shared: Behavior<Self::Shared>,
        ctx: &Self::Context,
    ) -> SceneBehavior<Self::Msg>;

    fn cursor_position(...) -> Option<(u16, u16)> { None }  // default
}
```

## The four type lanes

| Lane | Owner | Examples |
| ---- | ----- | -------- |
| `Context` | Host, pushed via `set_context` | participant identity, theme, capabilities |
| `Msg` | Host-defined per machine | `Submit`, `ScrollDown`, `ChunkReceived` |
| `Model` | The machine (participant-local) | focus, scroll offset, input buffer |
| `Shared` | Host projections, pushed via `set_shared` | transcript, roster, task board |

`Model` and `Shared` must implement `IntoGeometric` / `FromGeometric`
(re-exported from `knopper`) — the reactivity fingerprint used by the underlying
cliffy-core `Behavior`s. Hash-based encodings suffice; the native value is
cached and the geometric form is only a change fingerprint.

## init / update / project

- **`init(ctx) -> Model`** — initial participant-local state.
- **`update(&mut Model, Msg, &Context) -> Effect<Msg>`** — the only place
  `Model` changes. Returns an [`Effect`](../api/machine.md#effects), which the
  runtime drains synchronously.
- **`project(Behavior<Model>, Behavior<Shared>, &Context) -> SceneBehavior<Msg>`**
  — the scene as a *reactive function of state*. The runtime re-samples the
  projection whenever `Model` or `Shared` changes; there is no manual
  invalidation.

## PureMachine

For machines expressible as three closures, `PureMachine::new(init, update, view)`
(where `view` is the one-shot form of `project`) is the ergonomic path:

```rust
let machine = PureMachine::new(
    |_ctx: &MyCtx| MyModel::default(),
    |model, msg: MyMsg, ctx| { /* ... */ Effect::None },
    |model: &MyModel, shared: &MyShared, ctx: &MyCtx| {
        Scene::column(1_u64, vec![ /* ... */ ])
    },
);
```

Custom `Machine` impls remain available for machines that hold resources or
compose child machines with their own runtimes of state.

## Where to next

- [Scene Algebra](./scene-algebra.md) — what `project` returns
- [Collaboration: Shared and Model](./collaboration.md) — why the split exists
- [Writing a Machine](../guide/writing-a-machine.md) — step-by-step guide
