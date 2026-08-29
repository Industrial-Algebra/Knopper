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

## 0.2.0

- `collaboration` feature: Schubert capability seam, CRDT-backed incremental
  `Shared`
- Demo presence cues derived from a real `ParticipantRoster`
- Demo modules feature-gated or relocated (they are not stable API in 0.1.0)

## Downstream consumers

- **Wallace** — the collaborative AI harness; embeds Knopper per the validated
  contract. First slice: streaming transcript pane.
- **Tsume** — control panel; follows Wallace's embedding pattern.
- **Dominic** — cockpit; unblocked by the Tier 0 validation.

The working documents live in the repo: `docs/roadmap/` (milestone specs),
`docs/architecture/` (design chapters), `docs/handoff/` (session handoffs).
