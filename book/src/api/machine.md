# Machine & PureMachine

## `Machine`

The trait every UI unit implements. Four associated types — `Context`, `Msg`,
`Model`, `Shared` (see [The Machine Model](../concepts/machine-model.md)) —
and:

```rust
fn init(&self, ctx: &Context) -> Model;
fn update(&self, model: &mut Model, msg: Msg, ctx: &Context) -> Effect<Msg>;
fn project(
    &self,
    model: Behavior<Model>,
    shared: Behavior<Shared>,
    ctx: &Context,
) -> SceneBehavior<Msg>;
fn project_once(&self, model: &Model, shared: &Shared, ctx: &Context) -> Scene<Msg>; // provided
fn cursor_position(&self, model, shared, ctx, layout) -> Option<(u16, u16)>;          // default: None
```

Bounds: `Context: Clone`; `Model`/`Shared: Clone + IntoGeometric +
FromGeometric + 'static`; `Msg: Clone + 'static`.

## `PureMachine`

```rust
PureMachine::new(
    init:    impl Fn(&Context) -> Model,
    update:  impl Fn(&mut Model, Msg, &Context) -> Effect<Msg>,
    view:    impl Fn(&Model, &Shared, &Context) -> Scene<Msg>,
)
```

The closure form — `view` is the one-shot projection; `PureMachine` adapts it
to the reactive `project`. Covers most machines; implement `Machine` manually
when you need stored resources or non-closure logic.

## Geometric traits

`IntoGeometric` / `FromGeometric` / `GA3` are **re-exported from `knopper`**.
Std impls exist for numbers, `bool`, `String`, `()`, tuples, `Option<T>`. For
custom types, a hash-fingerprint impl suffices:

```rust
impl IntoGeometric for MyProjection {
    fn into_geometric(self) -> GA3 {
        // hash of contents in the scalar slot, length in e1 — the native
        // value is cached by Behavior; this is only a change fingerprint
        # /* see tests/embedding_harness.rs for a complete impl */
    }
}
```

`FromGeometric` is never used to reconstruct real values on the sampling
path; a placeholder is acceptable (as cliffy-core's own `String` impl does).

## `SceneBehavior`

The reactive scene type returned by `project` — a cliffy-core `Behavior`
specialization over scenes. The runtime re-samples it whenever `Model` or
`Shared` changes; you don't manage invalidation.
