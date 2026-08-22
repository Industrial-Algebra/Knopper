# Knopper Identity Restoration — 0.1.0 Re-cut Plan

**Date:** 2026-08-21
**Trigger:** RABBIT_HOLE_2026-08-21_Knopper.md + maintainer decision (B+).
**Status:** Release v0.1.0 is **deferred**. PR #9 closed; `release/v0.1.0` branch
kept for reference. Nothing has shipped; no downstream pinners. The version
remains ours to shape.

## The decision, verbatim from the maintainer

> B+, but keep and wire up rayon — the 80ms diff issue should also be addressed
> directly, and I think rayon might help there. Also, the GA was never intended
> to be ceremonial — it was foundational and central, like cliffy — that it got
> stubbed and shoved aside is the main thing I am upset about.

Therefore this plan's center is **making the geometry foundational again**, not
documenting its absence. The perf fix and the Schubert seam are real, but the
GA honesty is the identity.

## Verified findings (all checked against code, 2026-08-21)

- 11 `GA3::zero()` stubs across standard machines + demos; **zero readers** of
  the geometric form anywhere in `src/` (no `geometric_state`, no
  `apply_geometric`). Cliffy `Behavior` writes the multivector on every `set()`
  and nothing consults it; change notification is plain callbacks.
- `diff_render_ops` is O(n²) (linear `find` per op). Measured consequence:
  0.778 ms/append (release, n≈1000), projected ~80 ms/append at 10k lines.
- `rayon` is declared in Cargo.toml and imported nowhere.
- `demo.rs` (1,112 lines) ships as public API.
- `Presence.anchor: Option<NodeId>` collides with the planned viewport-bounded
  projection (anchors need rendered nodes to exist) — undesigned.
- `docs/release-0.1.0.md` checklist is stale (lists the already-absorbed
  `feature/focus-trap-semantics` merge as an open step).

## Work units (one session each, per agent discipline)

### Unit 1 — Streaming-append performance: O(n) diff + rayon (start here)

The landmine under the launch site; concrete, independently shippable.

1. **O(n) `diff_render_ops`**: HashMap/HashSet keyed by `NodeId`
   (`first-match-wins` insertion to preserve current `find()` semantics),
   single pass for inserts/updates in next-order, then removals in prev-order
   (patch order is asserted by existing tests — keep it identical).
2. **Wire rayon where it pays, with numbers**: parallel subtree recursion in
   `resolve_layout` / `render_ops` (`par_iter().map(...).collect()` — order
   preserved by indexed collect). Gate by size if small-scene overhead shows
   up. Validate with `tests/embedding_harness.rs`
   `s5_append_wall_clock_measurement` (before/after table goes in the PR).
3. Do **not** implement viewport-bounded projection yet — it collides with
   presence anchors (Unit 4 designs that seam first).

DoD: harness perf windows show sub-linear growth through 10k appends (extend
the loop if cheap); rayon used or removed-with-numbers; clippy/fmt/test green.

### Unit 2 — GA honesty: the identity work (the center)

1. **Encoding contract** (`docs/design/geometric-encoding-contract.md`):
   what every `IntoGeometric`/`FromGeometric` impl must satisfy —
   *injective* (required now), *semantically true* (required for types that
   will participate in distributed merge / capability arbitration: the
   collaboration lane first, host `Shared` projections by 0.2.0).
   Define blade assignments so encoders compose rather than ad-hoc hash.
2. **Replace every zero-stub** with structured, injective encodings — fields
   packed into named blades, documented per type.
3. **Collaboration types get semantically true encodings** aligned with
   roadmap 05: presence tones as distinct basis blades, `ParticipantId` in the
   scalar slot, roster as the multivector sum over its participants.
4. **Make geometry read, not just written** — at least one real consumer in
   0.1.0: derive display-meaningful data (e.g. presence tone aggregation for
   presence-slot annotation) *from the multivector*, demonstrating the
   GA→render path end-to-end. Candidate second consumer (cross-repo, propose
   not force): cliffy-core change-gated notification (notify subscribers only
   when the multivector changes — injective encodings make it sound; affects
   Borsalino et al., so it goes through cliffy's own gitflow).

DoD: zero `GA3::zero()` impls remain; encoding contract merged; a test proves
the GA-derived consumer changes output when (and only when) state changes.

### Unit 3 — Schubert capability seam (feature `collaboration`)

Pull roadmap 06 §5 forward behind a feature flag: `schubert` dep
(feature-gated), capability-gated activation (an `AccessController` consulted
in activation routing / projection of gated controls), per the verified 0.3.0
API (`AccessController::new(k,n)`/`register_capability`/`grant`/`check` →
`AccessDecision::{Granted, Impossible, Denied, Underconstrained}`).
Impossible-combination detection is the demo case for why geometry earns its
keep.

DoD: feature off by default; zero cost on default build; worked example test
(deploy-without-review rejection via `Impossible`).

### Unit 4 — Design debts + ghost sweep + re-cut

1. **Viewport × presence-anchor design** (`docs/design/`): always-rendered
   annotation lane vs. anchor exemption pass; decide before any viewport
   bounding lands.
2. Ghost sweep: feature-gate demo modules (`--features demo` or similar);
   refresh `docs/release-0.1.0.md` (checklist reality, new findings);
   reconcile mdbook design chapters with the new truth (GA chapter needed).
3. **Re-cut 0.1.0** from updated develop per ia-gitflow: new release branch,
   changelog gains the identity-pass sections, release PR, tag → publish
   (needs `CARGO_REGISTRY_TOKEN`) + Netlify docs deploy (needs
   `NETLIFY_SITE_ID`/`NETLIFY_AUTH_TOKEN` + site).

## Sequencing note

Units are ordered by risk-to-downstream: perf first (everyone hits it),
identity second (defines the API others encode against), seam third (depends
on the encoding contract), re-cut last. PR #10 (mdbook) can merge any time —
it is orthogonal and green.

## Non-goals (explicit)

- No viewport-bounded projection implementation before Unit 4's design.
- No cliffy-core changes inside this repo's PRs (propose upstream separately).
- No demo-API stability promises (still reference code; gating makes that
  honest).
