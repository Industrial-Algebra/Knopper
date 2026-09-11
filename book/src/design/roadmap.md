# Roadmap

## 0.1.0 (this release)

- Machine model + scene algebra + full rendering pipeline
- Principled focus: scene-derived order, scope policies, declarative modal
  trapping, disabled-node eligibility
- Standard machines (list, input, button, toggle, tabs, textarea,
  list-detail, command palette, modal helpers) + composition helpers
- Collaboration seam: `Shared`/`Model` split, `ParticipantRoster`, presence
  annotations
- Embedding contract validated (Tier 0); mock backend + integration harness
- Notcurses backend (feature-gated), Borsalino-pattern CI with a self-hosted
  notcurses runner

## 0.1.x

- **Append-optimized diff** — O(n) `diff_render_ops`; viewport-bounded
  projection (see [Streaming-Append](./streaming-append.md))
- Host-orchestration effects (contract §O3), if a pattern proves out

## Landed since the 0.1.0 roadmap was written

- **Geometric substrate made honest** (identity-restoration Units 1–3):
  encoding contract + blade ledger; all zero-stub encodings replaced;
  semantically true collaboration encodings (roster = multivector sum);
  the first GA→render reader (`tone_census`/`presence_annotation`)
- **`collaboration` feature: Schubert capability seam** (`CapabilityGate`
  + `CapabilityRuntime`, impossible-combination detection in Gr(2,4))
- **O(n) diffing** after the streaming-append hot path was measured and
  fixed; rayon tested and reverted on the numbers
- **Demo modules feature-gated** (`demo` feature; library-only default
  builds)
- **Viewport × presence-anchor design decided**: the annotation lane —
  see `docs/design/viewport-presence-anchor.md`

## 0.2.0

- CRDT-backed incremental `Shared`
- Demo presence cues derived from a real `ParticipantRoster`
- Viewport-bounded projection per the decided design (annotation lane +
  anchor-position resolution)
- GA-derived presence overlays beyond the status line (offscreen
  indicators, per the roadmap-03 local-visibility choices)

## Downstream consumers

- **Wallace** — the collaborative AI harness; embeds Knopper per the validated
  contract. First slice: streaming transcript pane.
- **Tsume** — control panel; follows Wallace's embedding pattern.
- **Dominic** — cockpit; unblocked by the Tier 0 validation.

The working documents live in the repo: `docs/roadmap/` (milestone specs),
`docs/architecture/` (design chapters), `docs/handoff/` (session handoffs).
