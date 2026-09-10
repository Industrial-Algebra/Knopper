# Acceptance testing in Knopper

**Status:** design (brainstormed 2026-08-28, operator-approved in three
sections). Target: **0.2.0** — explicitly out of scope for the 0.1.0 cut.

## Problem

Knopper's tests today are *structural*: they assert on models, scene
trees, and (primitively) render commands (`tests/runtime_pipeline.rs`
checks `backend.executed()` deltas). Nothing answers the acceptance
question — *a user who types these keys sees this screen* — and nothing
gives downstream embedders a way to write such tests for their own
machines. The demo feature-gating (Unit 4) sharpened this: the demos
are reference behavior with no executable specification.

## Decisions (from the brainstorm)

1. **Both, one toolkit.** The feature is downstream-facing — embedders
   test *their* machines — and Knopper's own demo journeys are its
   first consumer, which forces the API to be honest from day one.
2. **Assertions target what was drawn.** The acceptance currency is the
   backend layer, not the semantic scene. Concretely: the
   **post-application `BackendState`** (the screen model), *not* the
   executed-command deltas — deltas churn when the differ evolves
   (Unit 1 rewrote patch generation wholesale); the settled screen does
   not.
3. **Rust-native authoring.** Journeys are builder-chained Rust in
   `#[test]` functions — typesafe, refactorable, no parser or schema to
   maintain. Declarative script files are a possible later addition,
   not part of this design.

## Architecture

A new `acceptance` feature (off by default; zero default-build cost,
house rule) providing `knopper::acceptance`:

```rust,ignore
use knopper::acceptance::Journey;

Journey::new(machine, ctx, shared_initial)
    .resize(80, 24)                 // or set at construction
    .key(Key::Tab)                  // dispatch(RuntimeEvent::Key(..))
    .text("hello")                  // convenience: chars as key events
    .activate()                     // Enter on the focused node
    .set_shared(new_shared)         // host-lane push
    .render()                       // layout + lower + diff + apply to MockBackend

    // assertions on what was drawn (screen model):
    .assert_node_text(node_id, "expected text")
    .assert_node_absent(node_id)
    .assert_node_disabled(node_id)
    .assert_cursor(Some((row, col)))
    .assert_screen(&["line one", "line two"])   // golden text grid

    // white-box escape hatches (use sparingly):
    .state()    // &BackendState
    .model()    // M::Model
    .focus()    // &FocusState
```

`Journey<M>` owns a `Runtime<M>` and a `MockBackend`; every driver step
dispatches through the runtime like a real host (the same event
precedence the guides specify), and `render()` runs the full pipeline —
layout, lowering, diffing, backend apply. Nothing bypasses the seam an
embedder would use.

## Assertion surface

- **Primary: `BackendState`.** Per-node entries (text, rect, styles),
  cursor position, presence/absence. Assertions are phrased as settled
  screen facts: "node 7 shows *review: pending* at (2,3)". Because they
  read post-apply state, differ rewrites don't break them.
- **Golden screen grids.** A renderer projects `BackendState` to a text
  grid (the same projection a user reads). `assert_screen` compares
  against expected lines; failures dump **both grids plus a node table**
  — a failing journey shows the screen you got next to the screen you
  wanted. This dump is the debugging experience; it must exist before
  the first downstream consumer, not after.
- **Command-delta assertions: intentionally not offered** in the fluent
  surface (available via `.state()` escape hatch if ever needed). The
  diff stream is an implementation detail the tests should not date
  themselves to.

## Rollout

- **0.2.0**, behind `acceptance = []`. Downstream usage:
  `[dev-dependencies] knopper = { version = "0.2", features = ["acceptance"] }`.
- **First consumer: the demos.** `tests/demo_acceptance.rs`, gated
  `#![cfg(feature = "demo")]` + `--features demo,acceptance` lane:
  - *Workspace demo journey*: tab traversal across the shell, toggle
    activation, input commit, palette open/filter/commit, inspector
    toggle — asserting drawn screens at each checkpoint.
  - *Review demo journey*: query typing, draft editing + commit,
    follow toggle, publish gated on state — the full machine contract.
  Demo journeys double as executable documentation of intended
  behavior for the reference code.
- **CI**: a `demo,acceptance` test lane joins the hosted jobs (pure
  Rust, cheap); `demo,acceptance,notcurses` joins the self-hosted lane.

## Explicitly deferred

- Declarative scenario files (TOML/RON) — add only if non-Rust authors
  materialize; the Rust surface must stay complete without them.
- Image/screenshot goldens — the text grid covers terminal output.
- pty / real-terminal end-to-end runs — remains an ad-hoc smoke test
  (as done for notcurses), not part of the toolkit.
- Multi-participant journey orchestration (two runtimes, one roster) —
  natural 0.3 candidate once presence overlays render; the seam
  (`set_shared` + roster multivector readers) already exists for it.

## Non-goals

- No changes to `Runtime`'s dispatch semantics — the toolkit is a
  *client* of the existing seams, proving they suffice. If a journey
  needs a new runtime hook, that's a signal the embedding contract has
  a gap, and it goes through the contract process first.
- No performance assertions (the transcript harness owns that domain).
