# Tier 0 — Embedding-Contract Validation Spike

**Date:** 2026-08-17
**For:** the Knopper 0.1.0 release session (pre-release task — operator-planned).
**Blocks:** Wallace implementation + Tsume control panel. Unblocks Dominic M5's
cockpit.
**Contract:** `IA-documents/CONTRACTS/wallace-knopper-embedding.md` (v0.3).
**Context:** PULSE 2026-08-17 rec #4 — the contract is three versions ahead of the
code it constrains; its seams have never compiled against Knopper's actual
`machine.rs`/`runtime.rs`. This spike retires that drift risk before release.

## Goal

Prove (or correct) the embedding contract's §1–§5 assumptions against the real
runtime, on develop @ `15453fc` (all four 0.1.0 milestones). Every claim below is
**assessed from the 2026-08-11 code survey — verify each, don't assume it still
holds** after the PR #5 changes (especially the M2 collaboration module and the M3
runtime event policy).

## What the contract assumes (the checklist)

### §1 Embedding model — `Runtime` is embedder-driven

Verify this public surface exists and behaves as a host needs (the contract lists
it explicitly):

- `Runtime::new(machine, ctx, shared_initial)` / `set_context` / `set_shared`
- `dispatch(RuntimeEvent)` / `send(Msg)`
- `scene()` / `model()` / `shared()` / `focus()` / `layout(bounds)` /
  `render_ops(bounds)` / `cursor(bounds)` / `diff(bounds)`
- `render_to_backend*` / `invalidate_render_state`

**The non-goal to protect:** Knopper must not own a mandatory `run()` loop. If
PR #5's runtime event policy introduced any blocking/hosted loop, flag it — the
contract's §1 forbids displacing the embeddable `Runtime` (an *optional* hosted
loop for standalone apps is fine if feature-separated).

### §2 The drive loop — host-driven turn cycle

Write a minimal host harness (test or example, `mock` backend) that does exactly
the contract's loop: receive a key/resize → `dispatch`; receive a domain event →
`send`; projections changed → `set_shared` (+ `set_context` when app context
changes); read `diff(bounds)` or `render_to_backend`; commit. No event-loop
ownership from Knopper's side.

### §3 Projection-injection — the core seam (THIS IS THE SPIKE'S CENTER)

The contract's claim: **`Shared` = projection state pushed whole by the host via
`set_shared`; `Model` = participant-local UI state, machine-internal, never
pushed.** Specifically verify, with PR #5's M2 collaboration module in view:

1. Does `set_shared` trigger reprojection for a machine whose `project()` reads
   `Shared`? (The 08-11 survey said yes — `runtime_shared_state_updates_reproject_scene`
   test — confirm it survived the collaboration rework.)
2. Does the PR #5 collaboration seam preserve **`Model` = participant-local**? The
   collaboration-ready contract (roadmap doc 06) should agree with the embedding
   contract's §3/§7 that focus/selection/scroll live in `Model` and are never
   host-pushed. If the collaboration module moved any of that into `Shared` or a
   third location, **the two contracts disagree — reconcile them before 0.1.0**,
   since downstream (Wallace, Tsume panel) will pin against whichever wins.
3. Does `set_shared` (whole-projection replace) compose acceptably with the
   machine's own `update` on the same state? (Contract §O4 worries about
   granularity; the spike only needs "works," not "optimal.")

### §4 Message + effect contract

- Host `Msg`s are machine-typed; the adapter translates domain events → `send`.
  Verify a host can define its own `Msg` enum for a `PureMachine` without
  importing Knopper internals.
- Effects: what does `Machine::update`'s `Effect<Msg>` do on the runtime today?
  Document the actual semantics the host must know (does it self-dispatch
  follow-up msgs? need draining?). The contract's §O3 (async-effect bridge) stays
  open — this spike only records what exists.

### §5 Streaming-append requirement — the performance gate

The contract requires append-mostly scenes to stay cheap. Measure it:

- Build a transcript-like scene (list/textarea fed by a growing buffer), drive
  ~1,000 `set_shared` appends through `diff()`, record per-append cost.
- Success bar (from the contract): no O(n)-per-chunk re-layout of the whole
  scene; the visible window stable, the tail cheap.
- If it re-walks everything per chunk, that's the finding — document it as the
   0.1.x work item (an append-optimized diff path), not a contract change. The
  contract already anticipates this (§5 "if the current `diff` re-walks the whole
  scene per chunk, Knopper needs an append-optimized path").

## Deliverables

1. A `examples/` or `tests/` embedding harness exercising §1–§5 (the §5 bench can
   be a `#[test]` with rough timing assertions or an ignored perf test — house
   style is mock-backend tests).
2. **A short findings note** (`docs/embedding-validation-2026-08.md` or an update
   to the contract's §Change log via IA-documents PR): per-assumption ✓/✗, any
   contract corrections needed, the §5 measurement, and explicit sign-off on the
   `Model`/`Shared` question (item §3.2 above — the one that can disagree with
   the collaboration-ready contract).
3. Contract updates (if any) via PR to `IA-documents/CONTRACTS/` per its §9
   protocol — the doc is the source of truth, so corrections land there, not in
   prose in the Knopper repo.

## Definition of done

The Wallace build session can start against Knopper develop with the contract
marked **validated** (or with corrections merged), and the streaming-append cost
is a known number, not a guess.
