# The Geometric Substrate

Every machine state in Knopper carries a second form: alongside the typed
value (`ButtonState`, `InputState`, a whole `ParticipantRoster`) lives a
`GA3` multivector — an element of the 3D Euclidean geometric algebra
Cl(3,0). This chapter explains what that form is for, what it promises,
and what it deliberately does not promise.

The normative reference is
[`docs/design/geometric-encoding-contract.md`](https://github.com/Industrial-Algebra/Knopper/blob/develop/docs/design/geometric-encoding-contract.md)
in the repository; the `geometric` module's blade constants
(`SCALAR`..`E123`) and `Digest` are its Rust surface.

## Why state has a geometric form

The multivector is **not storage**. The typed state remains the single
source of truth. The geometric lane exists because three capabilities
need state in a form where *algebraic* operations are meaningful:

1. **Sound change detection.** The runtime substrate (cliffy `Behavior`)
   writes the multivector on every update. A change-gated notification —
   fire subscribers only when the geometry actually moved — is only
   correct if distinct states map to distinct multivectors. That is the
   contract's core demand: encodings are *injective as discriminants*,
   never lossy constants.
2. **Merge semantics.** Collaboration types encode so that *combination
   is addition* (below).
3. **Fingerprint derivation.** Downstream systems (reactive cells,
   Borsalino offload) derive fingerprints from these encodings as
   structured projections — compositional and documented, not hash
   garbage.

## The blade ledger

GA3 has 8 blades. Knopper assigns each a role; every non-zero
coefficient in every encoding must be attributable to a named field:

| Blade | Role |
|-------|------|
| `1` (scalar) | identity / count |
| `e1`, `e2`, `e3` | primary state axes; presence tones |
| `e12`, `e13`, `e23` | pairwise slots: markers, digest words |
| `e123` | aggregate / parity |

## Two classes of encoding

- **Class A — exact.** `from_geometric` inverts `into_geometric` on the
  nose. Required wherever the fields fit the ledger: `ButtonState`
  (scalar = activation count), `ToggleState`, `ListState`
  (selected + scroll), `TabsState` (with a marker blade distinguishing
  `Some(0)` from `None`).
- **Class B — structured discriminant.** Unbounded fields (strings,
  vecs, child states) cannot invert, so they pack *multiple structured
  components* — lengths, exact scalars, and the versioned `Digest`
  (SHA-256 into two 26-bit integer words) — into named blades.
  `InputState`'s text lives as a digest word pair plus cursor/length
  blades; aggregates like `CommandPaletteState` digest their children's
  *encodings*, so a child encoding change automatically updates every
  aggregate above it.

The contract's honesty clause: Class B is permitted **only** where
Class A is impossible, and `from_geometric` returns `Default` — the
typed cache is the reconstruction path, never the multivector.

## The collaboration lane: addition *is* the merge

The types that participate in distributed merging encode so the algebra
carries the semantics:

- `PresenceTone` occupies its own basis blade — `Local` → e1,
  `Collaborator` → e2, `Passive` → e3 — as a unit coefficient.
- A `Presence` is: one participant (scalar = 1), its tone blade, and
  digest words for `(id, anchor)`. Labels are excluded by design:
  presentation is not merge semantics, so relabeling never moves the
  geometry.
- **`ParticipantRoster` is the multivector sum over its participants.**

Because tones occupy disjoint blades, the sum cannot cancel or collide:
the scalar lands the participant count, and the tone blades carry an
*exact* tone census. Summation of exact integers is order-independent,
so the merged view is correct regardless of arrival order — the roster
merge is literally `+`.

## Geometry that reads: the presence annotation

The substrate would be ceremony if geometry were only ever written.
Knopper's reader path runs end to end in 0.1.0:

```rust,ignore
use knopper::collaboration::{presence_annotation, tone_census};

let mv = roster.into_geometric();          // the sum
let census = tone_census(&mv);             // exact counts off the blades
let text = presence_annotation(&mv);       // "you · 2 collaborators · 1 passive"
```

The annotation text feeds the existing `Annotation::PresenceSlot`
surface — display-meaningful data derived from the multivector alone,
never from the typed roster. Tests prove it changes when tone mix or
count changes and stays put when only labels do.

## What GA3 is not

GA3 is 8 floats: it is the *fingerprint layer*, not the place for
higher-grade mathematics. Grassmannian structure — Schubert cells,
grade-rich meet/join, capability arithmetic — exceeds GA3 by design and
lives in Schubert/amari territory. The boundary is concrete: the
capability seam (`collaboration` feature) consumes collaboration
identity at its edge and runs its own algebra; it never pushes results
back into GA3. See [Capability Gating](./capability-gating.md).
