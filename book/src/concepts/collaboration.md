# Collaboration: Shared and Model

Knopper's multi-user story is a type-level discipline, not a subsystem. The
`Machine` trait's `Shared` / `Model` split **is** the collaboration seam.

## The split

- **`Shared`** — projection state a machine projects from. The host computes
  projections (roster, transcript, task board, artifact index) from its own
  session state and pushes them **whole** via `Runtime::set_shared`. Multiple
  machines may consume slices of the same projection.
- **`Model`** — participant-local UI state (focus, scroll, selection, input
  buffer). Machine-internal; **never host-pushed, never synced wholesale.**

One `Runtime` per participant. Convergence across participants is the host's
problem (a CRDT or event-sourced core); Knopper receives already-merged
projections. This is deliberate: Knopper never sees a terminal byte stream and
never owns a network protocol.

## Canonical payload types

`knopper::collaboration` provides the vocabulary downstream apps share:

- `ParticipantId` — stable participant identity
- `Presence` / `PresenceTone` — presence state and its render tone
- `ParticipantRoster` — the canonical `Shared`-resident roster

Remote cursors and selections render as `Annotation::PresenceSlot` /
`Annotation::RemoteCursor` overlays — they annotate the scene without
entering layout or focus.

## What this buys a host

- **Focus stays honest.** Because focus is participant-local from day one,
  adding a second participant never invalidates the first's assumptions.
- **Machines are testable single-user.** `Shared` starts as `()` or a static
  projection; collaboration arrives by pushing real projections later.
- **The seam can't be painted shut.** Standard machines keep selection/commit
  in `Model`, so they're multi-user-safe by construction.

## Roadmap

The 0.1.0 seam is whole-projection push. Designed and mapped for 0.2.0:
CRDT-backed incremental `Shared` updates and the Schubert capability
integration (capability-gated actions per participant). See
[Collaboration-Ready Contract](../design/collaboration-contract.md).
