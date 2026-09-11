# Runtime Event Policy

How an event becomes a message: the dispatch precedence the runtime applies.

## Precedence

1. **Global shortcuts** — host/machine-declared global key handling
2. **Declarative Tab traversal** — Tab / Shift-Tab move focus across the
   scene-derived order, honoring scope policies (`Wrap` / `Trap` / `Local` /
   `Passthrough`)
3. **Machine `key_msg`** — the focused machine's raw-key handler gets a shot
4. **Declarative fallback** — activation (Enter on an `on_activate` node),
   focus events, ignore

The policy's design rule: **common behavior is declarative; machines opt into
raw keys deliberately.** Tab is not delivered to machines as a raw key unless
no declarative traversal applies — this is what makes modal trapping a scene
declaration instead of per-modal key filtering.

## RuntimeEvent

```rust
pub enum RuntimeEvent {
    Key(KeyEvent),      // key + ctrl/alt/shift
    Resize(ResizeEvent),
    Tick,
    Activate(NodeId),
    Focus(NodeId),
    Blur(NodeId),
    Sync(String),
}
```

`dispatch(event)` routes through the policy and, when routing yields a machine
message, applies it through `update` exactly like `send(msg)`. Focus-only
outcomes update `FocusState` without touching `Model`.

## Effects ride the same path

`Effect::Emit` re-enters `update` synchronously; `Effect::RequestFocus`
re-enters `dispatch`. One `send` can therefore cascade through a whole
message chain before returning — effects never queue for later and never
reach the host.

## For hosts

Translate your input layer into `RuntimeEvent` and nothing else. The demo's
`handle_runtime_key` (raw host) and the notcurses host both do exactly this —
see [Embedding Knopper](./embedding.md).
