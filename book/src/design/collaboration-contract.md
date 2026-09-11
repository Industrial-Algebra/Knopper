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

## Capability seam (landed, `collaboration` feature)

The Schubert capability seam shipped (identity-restoration Unit 3) behind
the `collaboration` feature against Schubert 0.5: `CapabilityGate`
(controller + node→requirement registry + `ParticipantId` bridge,
fail-closed decisions, `gated_scene` projection through the existing
`disabled` semantics) and `CapabilityRuntime` (activation gating with
`last_denial`). The headline capability is impossible-combination
detection — separation of duties as σ₂·σ₁₁ = 0, rejected with
`Impossible { conflicting }`. See
[Capability Gating](../guide/capability-gating.md).

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
