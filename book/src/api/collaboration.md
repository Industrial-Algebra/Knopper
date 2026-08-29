# Collaboration Types

`knopper::collaboration` — the canonical vocabulary for multi-participant
sessions. These types give downstream applications a shared `Shared`-payload
shape so presence renders consistently across hosts.

## `ParticipantId`

Stable identity for a session participant. Construct once per participant;
cheap to clone and compare.

## `Presence` and `PresenceTone`

A participant's presence state plus its render tone (how the UI should color
it). Presence is display metadata — it travels in `Shared` projections and
renders through annotation overlays.

## `ParticipantRoster`

The canonical roster: who is in the session, with their presence. Designed to
sit inside a host's `Shared` projection (e.g.
`struct SessionProjection { roster: ParticipantRoster, transcript: … }`).

## Annotations for presence

- `Annotation::PresenceSlot` — reserve a render slot for a participant's
  presence badge
- `Annotation::RemoteCursor` — a remote participant's cursor/selection overlay

These annotate the scene without affecting layout or focus — see
[Scene Algebra](../concepts/scene-algebra.md#annotations-and-presence).

## Design constraints

- One `Runtime` per participant; convergence is the host's responsibility
- Focus/scroll/selection never enter these types — they're `Model`-resident
- The demo's hardcoded presence cues are placeholders; 0.2.0 derives them from
  a real roster

See [Collaboration-Ready Contract](../design/collaboration-contract.md) for the
full specification and the Schubert capability seam mapped for 0.2.0.
