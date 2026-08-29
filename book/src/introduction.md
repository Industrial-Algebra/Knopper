# Knopper

> A functional-reactive terminal UI framework with a machine-centered programming
> model — designed multi-user from the ground up.

Knopper is a Rust framework for building terminal user interfaces as **machines**:
self-contained units that project a reactive `Scene` from local `Model` state and
replicated `Shared` state. A terminal-native **scene algebra** composes layout; a
single **rendering pipeline** lowers scenes to backend commands with structural
diffing. The Notcurses backend (behind a feature flag) targets high-performance
terminal rendering.

Knopper is the UI substrate for Industrial Algebra's collaborative tools — Wallace,
Dominic, and Tsume compose and render multi-participant sessions through it.

## Why Knopper?

- **Machine-first, not widget-first.** You implement one trait — `Machine` — with
  four associated types (`Context`, `Msg`, `Model`, `Shared`) and four functions
  (`init`, `update`, `project`, `cursor_position`). There is no widget tree to
  manage; the scene is a pure projection of state.
- **The Model/Shared split is the collaboration seam.** `Model` is
  participant-local (focus, scroll, input buffers — never synced). `Shared` is
  host-pushed projection state (rosters, transcripts, task boards). Multi-user
  support isn't bolted on; it's the type system.
- **Principled focus.** Focus order derives from the scene, traverses with scope
  policies (`Wrap` / `Trap` / `Local` / `Passthrough`), and modals trap focus
  declaratively — no imperative key routing.
- **Embeddable by contract.** Knopper owns no event loop and no process. A host
  drives `Runtime` turn-by-turn: `dispatch` keys, `send` messages,
  `set_shared` projections, commit `diff`s. The embedding contract is validated
  against the code (see [Embedding Contract](./design/embedding-contract.md)).

## A taste

```rust
use knopper::{Effect, PureMachine, Runtime, Scene};

let machine = PureMachine::new(
    |_ctx: &()| 0_i32,                                    // init
    |model: &mut i32, msg: bool, _ctx: &()| {             // update
        if msg { *model += 1; }
        Effect::None
    },
    |model: &i32, _shared: &(), _ctx: &()| {              // project
        Scene::text(1_u64, format!("count: {model}"))
            .on_activate(true)
    },
);

let mut runtime = Runtime::new(machine, (), ());
// Host drives turn-by-turn: dispatch events, read diffs, commit to a backend.
```

## Status

`0.1.0` is the first experimental release: coherent, tested (130 tests), and
clippy-clean — but unstable. Expect refinement in `0.2.0`, especially around the
collaboration seam and demo organization. See the
[roadmap](./design/roadmap.md).
