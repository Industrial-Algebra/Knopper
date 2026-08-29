# Collaboration-Ready Contract

The full specification is `docs/roadmap/06-collaboration-ready-contract.md` in
the repo. This chapter is the summary a host author needs.

## The architecture

- **External-merge seam.** Downstream hosts own convergence (CRDT, event
  sourcing, whatever) and push merged projections via `Runtime::set_shared`.
  Knopper never sees a wire protocol.
- **One Runtime per participant.** Each participant's process runs its own
  `Runtime` with its own `FocusState` and `Model`.
- **Presence as overlays.** Remote cursors/selections render through
  `Annotation::{PresenceSlot, RemoteCursor}` — never through focus.

## Capability seam (0.2.0, designed)

Per-participant capabilities (Schubert `AccessController`) map onto the
embedding contract's `Context` lane: the host pushes the participant's
capabilities in `Context`, and machines consult them when projecting
capability-gated controls (disabled vs. hidden per `PresenceTone`). The exact
`AccessController` 0.3.0 API surface was verified against Schubert's code; the
integration lands behind a `collaboration` feature in 0.2.0.

## Checklist for a collaboration-ready machine

1. Is each piece of state local (`Model`), shared (`Shared`),
   participant-local-published (presence annotation), or derived (scene)?
2. Does anything assume a single focus or a single selection? (It must not.)
3. Are activation/commit paths expressible per-participant?
4. Do ids stay stable across projection updates?

## Status

0.1.0: the seam, the canonical types, and the audit are done; demos still use
cosmetic presence. 0.2.0: real roster-driven presence in the demos, the
`collaboration` feature, incremental `Shared` updates.
