# Changelog

All notable changes to Knopper are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.0] — Unreleased

First experimental release. The API is coherent and tested but should be
treated as unstable until 0.2.0.

### Added — Machine-first core
- **`Machine` trait** — UIs as machines projecting `Behavior<Scene<Msg>>`
  from local `Model` and replicated `Shared` state
- **`PureMachine`** for closure-defined machines; custom impls for richer
  cases
- **Full rendering pipeline** — runtime event routing → update → effect →
  reprojection → layout → render lowering → diffing → backend commands
- **Mock backend** for tests; Notcurses backend behind the `notcurses`
  feature

### Added — Scene algebra
- Composition combinators: column/row/stack, padding, border, sized,
  viewport, scroll, align, annotated
- **`FocusScope`** with explicit policies (`Wrap` / `Trap` / `Local` /
  `Passthrough`)
- **`NodeMeta.disabled`** + `Scene::disabled()` builder — disabled nodes
  are skipped by focus collection and activation routing

### Added — Principled focus model
- Scene-derived focus order, scope-aware Tab traversal
- Modal trapping via declarative `Trap` policy (clamps at boundaries,
  never wraps)
- Focus is participant-local by design

### Added — Standard machines
- list, input, button, toggle, tabs, textarea, list-detail, command
  palette, modal helper
- Composable via shared helpers in `compose`

### Added — Geometric substrate made honest (identity restoration, Unit 2)
- **Encoding contract** (`docs/design/geometric-encoding-contract.md`) —
  the blade ledger over GA3's 8 coefficients, Class A (exact round-trip) vs
  Class B (structured discriminant) encoding rules, and the bridge that
  keeps GA3 as the fingerprint layer while higher-grade math lives at the
  amari/Borsalino seam
- **`geometric` module** — blade constants (`SCALAR`..`E123`) and the
  versioned `Digest` (SHA-256, two 26-bit integer words chosen so
  multivector sums stay exact and order-independent)
- **All 11 zero-stub `IntoGeometric` impls replaced** — every machine
  state now encodes into named blades per the contract register (exact
  for button/toggle/list/tabs; structured discriminants for
  input/textarea/list-detail/palette/demos; structural fingerprint for
  `Scene`)
- **Semantically true collaboration encodings** — presence tones are unit
  coefficients on their own basis blades (`Local`→e1, `Collaborator`→e2,
  `Passive`→e3), and `ParticipantRoster` is the multivector sum over its
  participants: roster merge *is* addition, order-independent by
  construction
- **The first geometry reader** — `tone_census` / `presence_annotation`
  derive display-meaningful presence-slot content from the multivector
  alone (GA→render path, exercised end-to-end in
  `tests/geometric_reader.rs`)

### Added — Collaboration-ready contract
- **`ParticipantId` / `Presence` / `ParticipantRoster`** — canonical
  `Shared` payload for multi-user sessions
- One `Runtime` per participant; shared state flows in via
  `Runtime::set_shared`
- Remote cursors/selections as annotation overlays
  (`Annotation::PresenceSlot`, `Annotation::RemoteCursor`)
- Schubert capability integration designed and mapped for 0.2.0
  (`docs/roadmap/06-collaboration-ready-contract.md`)

### Added — Documentation and policy
- `docs/guides/08-runtime-event-policy.md` — dispatch precedence
  (global shortcuts → declarative Tab → machine key_msg → declarative
  fallback)
- Integration tests against the mock backend (`tests/runtime_pipeline.rs`)
- Two runnable demos (interactive workspace host, review-focused
  workspace)

### Fixed
- **`FocusScopePolicy::Trap` semantics** — previously cycled like `Wrap`
  in `FocusNavigation::advance`; now clamps/returns `None` at boundaries
- Modal migrated off imperative Tab handling onto declarative runtime
  focus

### CI / infrastructure
- Split core gate (fmt/clippy/test/doc on default features) from the
  notcurses feature lane
- Self-hosted `notcurses` runner (system notcurses 3.0.17) for feature
  verification, gated on release PRs / `run-notcurses` label
- Borsalino-conformant branch protection and release workflow

### Not stable in 0.1.0
- Demo modules (`demo`, `demo_ui`, `review_demo`) are public for
  reference but are not stable API
- Expect refinement in 0.2.0, especially around the collaboration seam
  and demo organization

