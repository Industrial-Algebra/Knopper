# Writing a Machine

A step-by-step walk from empty type to working machine.

## 1. Name your four types

```rust
#[derive(Clone)]
struct Ctx { user: String }          // Context — pushed via set_context

#[derive(Clone)]
enum Msg { Submit, Cancel }          // Msg — host/machine vocabulary

#[derive(Clone, Default)]
struct Form { buffer: String }       // Model — participant-local

type Shared = ();                    // Shared — host projection (none yet)
```

`Model` and `Shared` need `IntoGeometric`/`FromGeometric` (re-exported from
`knopper`). Numbers, `bool`, `String`, `()`, tuples, and `Option` implement it
out of the box; for structs, a hash-based impl is fine — the native value is
cached and the geometric form is only a change fingerprint.

## 2. init — initial local state

```rust
|ctx: &Ctx| Form::default()
```

## 3. update — the only place Model changes

```rust
|model: &mut Form, msg: Msg, _ctx: &Ctx| match msg {
    Msg::Submit => { model.buffer.clear(); Effect::None }
    Msg::Cancel => { model.buffer.clear(); Effect::None }
}
```

Effects are for control flow, not orchestration: `Effect::Emit(msg)` chains a
follow-up message synchronously, `Effect::Batch` applies several,
`Effect::RequestFocus(id)` moves focus. The runtime drains them completely —
see [Runtime](../api/runtime.md#effects).

## 4. project — the scene as a function of state

```rust
|model: &Form, _shared: &(), ctx: &Ctx| {
    Scene::column(1_u64, vec![
        Scene::text(2_u64, format!("hello, {}", ctx.user)),
        Scene::text(3_u64, model.buffer.clone()),
        Scene::text(4_u64, "[ submit ]")
            .focusable()
            .on_activate(Msg::Submit),
    ])
}
```

Use stable ids; the diff is id-keyed. For `PureMachine` this closure is the
`view` argument; custom `Machine` impls get the reactive `project` form with
`Behavior` arguments.

## 5. Drive it

```rust
let mut runtime = Runtime::new(machine, Ctx { user: "ada".into() }, ());
runtime.send(Msg::Submit);
let mut backend = knopper::MockBackend::default();
runtime.render_to_backend(&mut backend, Rect::new(0, 0, 80, 24)).unwrap();
```

## Testing

Machines are plain values — test `update` directly, and test the runtime path
against `MockBackend`. `tests/embedding_harness.rs` in the repo is a complete
worked example of host-side machine definitions and assertions.
