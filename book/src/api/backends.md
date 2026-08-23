# Backends

Knopper renders to a `TerminalBackend`. Two ship today; the trait is the
extension point (a future web/Sixel backend implements the same surface).

## `TerminalBackend`

```rust
pub trait TerminalBackend {
    type Error;
    fn execute(&mut self, commands: &[BackendCommand]) -> Result<(), Self::Error>;
    fn sync_order(&mut self, order: &[NodeId]) -> Result<(), Self::Error>;
}
```

`execute` applies the translated patch commands; `sync_order` communicates the
front-to-back stacking order (for backends with real z-ordering, like
notcurses planes). The runtime calls both on every commit.

## `MockBackend`

In-memory backend that records executed commands and exposes a queryable
`BackendState` (entries by `NodeId`, cursor, surface count). This is the test
backend — the entire CI suite runs against it, and host test harnesses should
too.

Also available: `MockRenderer` for the simpler `Renderer` trait path.

## `NotcursesBackend` (feature `notcurses`)

Real terminal rendering via notcurses planes — one plane per scene node,
synced in render order. Requires the **system notcurses library ≥ 3.0.11**
(`libnotcurses-dev` on Ubuntu 26.04+; note Ubuntu 24.04's 3.0.7 is too old).
The crate's `vendored` cargo feature is a docs-only path and does not link —
install the system library.

```toml
knopper = { version = "0.1", features = ["notcurses"] }
```

## Supporting types

- `BackendCommand` — the instruction set (create/update/remove surface, draw
  text/border, set cursor, …)
- `BackendEntry` / `BackendState` — recorded backend state (mock inspection)
- `backend_commands(previous, patches)` — the patch → command translation,
  public for custom backends
