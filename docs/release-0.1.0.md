# Knopper 0.1.0 release preparation

> **Release shaping document.** Status of the 0.1.0 experimental release,
> API surface review, the cut checklist, and draft release notes.
>
> Companion to [roadmap/00-first-release-roadmap.md](roadmap/00-first-release-roadmap.md).

## Status: ready for the 0.1.0 cut

The four pre-release milestones are substantively complete:

| Milestone | Status | Delivered |
|---|---|---|
| M1 — focus scope policies | ✅ Done | Corrected `FocusScopePolicy::Trap` semantics (clamp, not wrap); modal migrated to declarative runtime focus; `disabled` node eligibility; vestigial imperative focus helpers removed |
| M2 — collaboration-readiness | ✅ Done | `collaboration` module (`ParticipantId`/`Presence`/`ParticipantRoster`); code-grounded contract doc; Schubert 0.3.0 API verified and mapped for the 0.2.0 capability seam |
| M3 — runtime event policy + tests | ✅ Done | Dispatch-precedence guide; first integration test crate (`tests/runtime_pipeline.rs`) characterizing the full pipeline |
| M4 — release shaping | ✅ This doc | License headers + LICENSE + Cargo metadata; API surface review; release notes |

Quality gate (must be green at cut):

- [x] `cargo fmt --all`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test --all-features` — 119 unit + 5 integration tests pass

## API surface review

The public API is exported from `src/lib.rs`. It is coherent and
intentionally shaped, with one structural caveat.

### Coherent areas

- **Core model:** `Machine`, `PureMachine`, `SceneBehavior`, `Effect`,
  `Scene`, `NodeMeta`, `NodeId` — the machine-first core.
- **Pipeline:** `Runtime`, `route_event`, `RoutedEvent`, `LayoutNode`,
  `resolve_layout`, `render_ops`, `RenderOp`, `diff_render_ops`, `PatchOp`,
  `Renderer`/`MockRenderer`, `TerminalBackend`/`MockBackend`/`BackendCommand`.
- **Focus:** `FocusState`, `FocusPath`, `FocusOrder`, `FocusNavigation`,
  `FocusScopePolicy`, plus compose helpers (`child_has_focus`,
  `dispatch_if_focused`, `trap_focus`, `update_child`, `project_child`,
  `map_effect`).
- **Standard machines:** list, input, button, toggle, tabs, textarea,
  list_detail, command_palette, modal — each with `Context`/`Machine`/
  `Msg`/`State` and a `*_key_msg` helper where relevant.
- **Collaboration:** `ParticipantId`, `Presence`, `PresenceTone`,
  `ParticipantRoster` — the canonical `Shared` payload for collaborative
  machines.

### Caveat: demo modules are public

`demo`, `demo_ui`, and `review_demo` are `pub mod` and re-exported
(`DemoMachine`, `ReviewDemoMachine`, etc.). This makes in-tree demo code
part of the library's public API surface, which downstream projects could
come to depend on.

**Decision for 0.1.0:** ship as-is. The demos are useful as runnable
reference and composition templates, and cutting them now adds risk. Flag
in release notes that demo modules are **not** stable API and may move or
be feature-gated in 0.2.0. Track as the first 0.2.0 cleanup task:
feature-gate behind an `examples` or `demo` feature, or move under
`examples/`.

### Minor naming notes (non-blocking)

- `PresenceTone` exists in both `collaboration` (canonical semantic type)
  and `demo_ui` (presentation projection). Acceptable — they live in
  different modules — but worth a doc cross-reference so readers know which
  is which. The contract doc §4 covers this.
- `Presence` is a fairly generic name at the crate root. Acceptable for
  0.1.0 given the collaboration focus; revisit if name collisions bite.

## 0.1.0 cut checklist

### Must pass before tag

- [x] All four milestones substantively delivered
- [x] License headers on every `.rs` file (Apache-2.0)
- [x] `LICENSE` file present (Apache-2.0)
- [x] `Cargo.toml` declares `license`, `repository`, `description`
- [x] Quality gate green (fmt / clippy -D warnings / test)
- [ ] **Confirm licensing choice with maintainer** (Apache-2.0 assumed per
      IA standard for framework crates and matching the Schubert dependency;
      see `ia-licensing` skill)
- [ ] **Establish `develop` branch** per IA gitflow (CONTRIBUTING prescribes
      topic → develop → release PR → main; currently only `main` exists)
- [ ] Merge `feature/focus-trap-semantics` into `develop` via PR
- [ ] Open release PR `develop` → `main` with this doc and the change log
- [ ] Tag `v0.1.0` on `main`
- [ ] Verify `cargo doc --no-deps --all-features` builds clean

### Should have before tag

- [ ] Notcurses interactive demo smoke-tested in a real terminal (cannot be
      done in a non-interactive session; the integration tests + mock
      backend cover the framework contract, but real Notcurses rendering
      needs a human-in-the-loop check)
- [ ] Cross-reference `PresenceTone` between `collaboration` and `demo_ui`
      in docs

### Deferred to 0.2.0 (explicitly out of 0.1.0 scope)

- Feature-gate or relocate demo modules out of the public API
- Schubert capability seam (`collaboration` feature flag) — designed in
  [roadmap/05](roadmap/05-distributed-correctness-schubert.md) and
  [06](roadmap/06-collaboration-ready-contract.md) §5
- CRDT-backed incremental `Shared` merge (replacing wholesale `set_shared`)
- Demo refactor to derive presence cues from a real `ParticipantRoster` in
  `Shared` (proves the collaboration seam end-to-end)
- Multi-agent coding harness (roadmap 05 Phase 3)

## Known gaps (documented, not blocking)

1. **No `tests/` coverage of Notcurses backend.** The Notcurses backend is
   feature-gated and needs a real terminal; integration tests use the mock
   backend. Backend correctness is exercised via the same command path.
2. **Demo `Shared = ()`.** The in-tree demos do not yet exercise the
   collaboration `Shared` lane end-to-end. The `collaboration` module and
   contract doc give downstream the template; a demo refactor is tracked as
   a 0.2.0 item.
3. **Gitflow not yet established.** Commits currently land on `main`
   directly. CONTRIBUTING documents the intended flow; `develop` should be
   created at release time.

## Draft release notes

### Knopper 0.1.0

Knopper is a Rust framework for building functional-reactive terminal user
interfaces with a machine-centered programming model, a terminal-native
scene algebra, and a rendering pipeline tailored to high-performance
Notcurses backends. It is designed multi-user from the ground up as a
collaboration substrate for downstream Industrial Algebra projects.

This is the first experimental release. The API is coherent and tested but
should be treated as unstable until 0.2.0.

#### Highlights

- **Machine-first core.** Build UIs as `Machine` implementations that
  project `Behavior<Scene<Msg>>` from local `Model` and replicated `Shared`
  state. `PureMachine` for closures, custom impls for richer cases.
- **Scene algebra.** Compose layouts with column/row/stack, padding, border,
  sized, viewport, scroll, align, annotated, and `FocusScope` with explicit
  policies (`Wrap` / `Trap` / `Local` / `Passthrough`).
- **Full rendering pipeline.** Runtime event routing → update → effect →
  reprojection → layout → render lowering → diffing → backend commands.
  Mock backend for tests; Notcurses backend behind the `notcurses` feature.
- **Principled focus model.** Scene-derived focus order, scope-aware Tab
  traversal, modal trapping via declarative `Trap` policy, `disabled` node
  eligibility. Focus is participant-local by design.
- **Standard machines.** list, input, button, toggle, tabs, textarea,
  list-detail, command palette, modal helper — composable via shared
  helpers in `compose`.
- **Collaboration-ready.** `ParticipantId` / `Presence` /
  `ParticipantRoster` give downstream apps a canonical `Shared` payload for
  multi-user sessions. One `Runtime` per participant; shared state flows in
  via `Runtime::set_shared`. Schubert capability integration is designed
  and mapped for 0.2.0.
- **Two runnable demos.** A main interactive workspace host and a
  review-focused workspace, exercising the standard machines and
  composition helpers.

#### Getting started

```bash
./scripts/setup-hooks.sh
cargo run                              # raw-key interactive host
cargo run --features notcurses -- --notcurses
cargo run -- --demo review
```

#### Quality

124 tests (119 unit + 5 integration), `clippy -D warnings` clean across all
feature combinations, `cargo fmt` clean. Apache-2.0 licensed.

#### Not stable in 0.1.0

- Demo modules (`demo`, `demo_ui`, `review_demo`) are public for reference
  but are not stable API.
- Any item not documented as stable in the guides. Expect refinement in
  0.2.0, especially around the collaboration seam and demo organization.

---

*Release preparation document for Knopper 0.1.0.*
