# Runtime

`Runtime<M: Machine>` is the embedder-driven engine: it owns the `Behavior`s
for `Model` and `Shared`, the projected `SceneBehavior`, the `FocusState`, and
the last committed frame. One `Runtime` per participant.

## Construction

```rust
Runtime::new(machine, ctx, shared_initial)
```

## Driving (host turn cycle)

| Method | Use |
| ------ | --- |
| `dispatch(RuntimeEvent)` | keys, resize, focus/activate events |
| `send(Msg)` | domain messages from the host adapter |
| `set_shared(Shared)` | push a whole projection (re-projects on next sample) |
| `set_context(Context)` | push app context (identity, theme) |

## Reading

| Method | Returns |
| ------ | ------- |
| `scene()` | `&SceneBehavior<Msg>` |
| `model()` / `shared()` | current values (cloned) |
| `focus()` | `&FocusState` |
| `layout(bounds)` | `LayoutNode` tree |
| `render_ops(bounds)` | `Vec<RenderOp>` |
| `cursor(bounds)` | `Option<(u16, u16)>` (clamped to bounds) |
| `diff(bounds)` | `Vec<PatchOp>` vs. last commit — **read-only** |

## Committing

| Method | Notes |
| ------ | ----- |
| `render(renderer, bounds)` | generic `Renderer` path |
| `render_to_backend(backend, bounds)` | diff → backend commands → execute → sync order |
| `render_to_backend_with_cursor(.., cursor)` | + explicit cursor command |
| `render_to_backend_auto_cursor(..)` | cursor from the machine |
| `invalidate_render_state()` | forget the baseline (next commit re-inserts everything) |

## Effects

`Machine::update` returns `Effect<Msg>`:

```rust
pub enum Effect<Msg> {
    None,
    Emit(Msg),              // synchronously re-enters update (recursive)
    Batch(Vec<Effect<Msg>>),// applied in order
    RequestFocus(NodeId),   // routes through the focus dispatch path
}
```

Effects are **closed and fully drained** by the runtime inside
`send`/`dispatch`. There is no queue, no host-visible effect stream, and no
async bridge (yet — that's an open design question tracked in the embedding
contract's §O3).

## Threading

`Runtime` is single-threaded (`Rc`/`RefCell` inside the Behaviors). Drive it
from one task; do async work elsewhere and surface results as `send` /
`set_shared`.
