# Getting Started

Knopper is a Rust library. Add it to your project:

```toml
[dependencies]
knopper = "0.1"
```

The default build is backend-agnostic — you get the full machine/scene/runtime
model plus the mock backend used for tests. For the Notcurses terminal backend:

```toml
[dependencies]
knopper = { version = "0.1", features = ["notcurses"] }
```

> **System dependency.** The `notcurses` feature links the system notcurses
> library and requires **notcurses ≥ 3.0.11** (`libnotcurses-dev` on
> Debian/Ubuntu 26.04+, or your distro's equivalent). The Rust `vendored`
> feature is a docs-only path and does not produce a linkable library.

## Your first machine

```rust
use knopper::{Effect, PureMachine, Runtime, RuntimeEvent, Scene, Rect};

#[derive(Clone)]
enum Msg { Bump }

let machine = PureMachine::new(
    |_ctx: &()| 0_i32,
    |model: &mut i32, msg: Msg, _ctx: &()| match msg {
        Msg::Bump => { *model += 1; Effect::None }
    },
    |model: &i32, _shared: &(), _ctx: &()| {
        Scene::text(1_u64, format!("bumps: {model}")).on_activate(Msg::Bump)
    },
);

let mut runtime = Runtime::new(machine, (), ());
let bounds = Rect::new(0, 0, 80, 24);

// One host turn: send a message, read the paint, commit it.
runtime.send(Msg::Bump);
let patches = runtime.diff(bounds);      // structural patch vs. last commit
let mut backend = knopper::MockBackend::default();
runtime.render_to_backend(&mut backend, bounds).unwrap();
```

From here:

- [Writing a Machine](./guide/writing-a-machine.md) — the full trait and
  state lanes
- [Composing Machines](./guide/composing-machines.md) — building screens from
  standard machines
- [Running the Demos](./guide/demos.md) — two runnable reference applications
- [Embedding Knopper](./guide/embedding.md) — driving `Runtime` from your own
  event loop (the Wallace pattern)

## Development setup

```bash
git clone https://github.com/Industrial-Algebra/Knopper.git
cd Knopper
./scripts/setup-hooks.sh
cargo test                 # default features — the library contract
cargo test --features notcurses   # needs system notcurses ≥ 3.0.11
```
