# Geometric encoding contract

**Status:** Unit 2 of the identity-restoration plan
(`docs/plans/2026-08-21-identity-restoration.md`). Governs every
`IntoGeometric` / `FromGeometric` impl in Knopper. Rust surface lives in
`src/geometric.rs`; this document is normative.

## 1. What the geometric lane is for

The `GA3` multivector attached to every machine state is **not storage**.
The typed state is the source of truth; the multivector is the substrate
view — the form on which three capabilities operate:

1. **Sound change detection.** cliffy's `Behavior` writes the multivector
   on every `update`; a future change-gated notification (cliffy upstream
   candidate) fires subscribers only when the multivector actually
   changes. That gate is only correct if distinct states map to distinct
   multivectors.
2. **Merge semantics.** Distributed-merge participants (the
   collaboration lane, host `Shared` projections by 0.2.0) need state in
   a form where algebraic combination is *meaningful*, not just
   collision-free.
3. **Fingerprint derivation.** cliffy reactive-cell fingerprints and
   future Borsalino hand-offs derive from these encodings as
   structured projections — compositional, documented, reproducible —
   never ad-hoc hash garbage.

## 2. The blade ledger

`GA3 = Multivector<3,0,0>` (amari) has 8 blades, coefficient indices
fixed by amari's Hadamard ordering. Knopper assigns them **roles**; every
encoding must respect the ledger:

| Index | Blade | Role |
|-------|-------|------|
| 0 | `1` (scalar) | identity / count — participant count, node count, small primary scalars |
| 1 | `e1` | primary state axis A — first numeric field; collaboration: `Local` tone |
| 2 | `e2` | primary state axis B — second numeric field; collaboration: `Collaborator` tone |
| 3 | `e3` | primary state axis C — third numeric field / marker bit; collaboration: `Passive` tone |
| 4 | `e12` | pairwise slot A — relation/commit marker or digest word 1 |
| 5 | `e13` | pairwise slot B — relation/commit marker or digest word 2 |
| 6 | `e23` | pairwise slot C — relation/commit marker or digest word 3 |
| 7 | `e123` | pseudoscalar — aggregate / parity / overflow marker |

Roles are vocabulary, not straitjackets: a type may repurpose a blade if
the per-type entry (§5) documents it and no *composition* collides.
What the ledger forbids is unstructured packing — every non-zero
coefficient must be attributable to a named field.

## 3. Two classes of encoding

GA3 is 8 `f64`s. Honest about what fits:

### Class A — exact
`FromGeometric` inverts `IntoGeometric` on the nose. Required whenever
the encoded fields' total information fits the ledger (bools, `u16`s,
`usize` selections, small enums). Exactness bound: integers are exact in
`f64` up to 2^53; states whose counters can provably exceed 2^53 (none
today) must document the rounding or move to Class B.

### Class B — structured discriminant
For fields whose domain is unbounded (strings, vecs, node lists): the
encoding packs **multiple structured components** — length, canonical
digest (SHA-256, two 52-bit words split across assigned blades), and
semantic scalars — into the ledger. Properties:

- **Change-sound, not invertible:** distinct states map to distinct
  multivectors up to SHA-256 collision; `FromGeometric` returns
  `Default` (reconstruction comes from the typed cache, as cliffy's own
  `String` impl acknowledges). Never claimed as a round-trip.
- **Compositional:** aggregates encode children through the children's
  own encodings — digest the child's multivector canonical form into a
  word pair — rather than re-deriving from raw fields. A child encoding
  change automatically updates every aggregate.
- **Versioned:** `Digest` (src/geometric.rs) fixes the algorithm
  (SHA-256 over the canonical coefficient array, two words: lo/hi 26+26
  bits, offset-biased to avoid −0.0/NaN quirks). Changing it is a
  contract break and must not happen silently.

**Class B is permitted only where Class A is impossible.** A field that
fits the ledger must be encoded exactly, never hashed.

## 4. The collaboration lane — semantically true

The types that participate in distributed merge get encodings where the
*algebra* carries the semantics (roadmap 05 alignment):

- **`PresenceTone`** — a tone is a **unit coefficient on its own basis
  blade**: `Local → e1`, `Collaborator → e2`, `Passive → e3`. Tones are
  *mixable* by multivector addition: summing presences yields tone
  counts exactly (disjoint blade supports ⇒ no cancellation, no
  collision).
- **`Presence`** — `scalar = 1` (one participant), the tone blade set
  to 1, `e12/e13 = Digest(id ‖ anchor)` (length-framed id bytes, an
  anchor-present marker byte, and `NodeId`'s raw u64 when set).
  `FromGeometric` returns an explicit empty placeholder — `Presence`
  has no `Default` because a participant id cannot be defaulted
  honestly. Label does **not** participate: labels are presentation,
  not merge semantics.
- **`ParticipantRoster`** — **the multivector sum over its
  participants' presence encodings.** Consequently:
  - `scalar` = participant count,
  - `|e1|`, `|e2|`, `|e3|` = exact tone census (Local / Collaborator /
    Passive counts),
  - identity words are a commutative digest-sum (order-independent —
    correct for a merged roster view).

This is the property the standard-machine encodings do **not** claim:
roster combination is *addition*, and addition is the merge.

## 5. Per-type register

Every `IntoGeometric` impl in the crate, its class, and its ledger use.
(Fragment states: `demo.rs` / `review_demo.rs` are binaries-adjacent
demo aggregates; they follow the aggregate rule like `CommandPalette`.)

| Type | Class | Encoding |
|------|-------|----------|
| `ButtonState` | A | `1 = activations` (exact below 2^53) |
| `ToggleState` | A | `1 = checked` (0/1) |
| `ListState` | A | `1 = selected`, `e1 = scroll` (u16 exact) |
| `TabsState` | A | `1 = selected`, `e1 = committed.value`, `e12 = committed.marker` (`Some(0)` ≠ `None` by marker) |
| `InputState` | B | `e1 = cursor`, `e2 = value.len`, `e12/e13 = Digest(value)`, `e23/e123 = Digest(committed)` (`None` = reserved zero pair); scalar reserved |
| `TextareaState` | B | `1 = preferred.value`, `e1/e2 = cursor row/col`, `e3 = preferred.marker`, `e12/e13 = Digest(value)`, `e23/e123 = Digest(committed)` |
| `ListDetailState` | B | aggregate: `1/e1` = child `ListState` exact pair, `e2/e12` = committed value/marker, `e13/e23 = Digest(child encoding)` |
| `CommandPaletteState` | B | aggregate: `1 = filtered.len`, `e1 = open`, `e2/e3 = committed value/marker`, `e12/e13 = Digest(input child)`, `e23/e123 = Digest(list child)` |
| `DemoState` | B | aggregate: `1 = task_done true-count`, `e1 = inspector`, `e2 = focus declared`, `e3 = cursor present`, `e12/e13 = Digest(child encodings: tabs, toggle, button, textarea, list, palette)`, `e23/e123 = Digest(string fields: status, last_input)` |
| `ReviewDemoState` | B | aggregate: `e1 = focus declared`, `e2 = cursor present`, `e12/e13 = Digest(child encodings: tabs, query, draft, list, follow, publish)`, `e23/e123 = Digest(status)`; scalar reserved |
| `Scene<Msg>` | B | `1 = node count`, `e1/e2 = Digest(structural walk)` — v1 walk covers kind tag, node id, text content, focus-scope name, annotated label, padding, scroll offset; styles, roles, annotations, interactions, policies, constraints and anchor *values* are excluded (geometry changes are what the substrate gates on; widen deliberately, never silently) |
| `Presence` | true | §4 |
| `ParticipantRoster` | true | §4 (sum; empty roster = empty sum = zero, legitimately) |

## 6. The reader — geometry must be read, not just written

At least one real consumer in 0.1.0 (DoD): **presence-slot annotation**
derives its displayed content *from the roster multivector*:

```text
roster → GA3 (sum) → tone census (blade coefficients) → annotation text
```

`tone_census(&GA3) -> ToneCensus` reads `scalar` and the three tone
blades exactly. The annotation shows counts ("2 collaborators, 1
passive"); the data path is multivector → coefficients → display, end
to end. This is deliberately *not* the same as reading the typed roster:
the point is proving the GA→render path. (Long-term readers — viewport×
presence-anchor geometry in Unit 4 — build here.)

## 7. The bridge — what exceeds GA3

Grassmannian structure (Schubert cell arithmetic, grade-rich subspace
meet/join) exceeds GA3 **by design**. The bridge:

- GA3 remains the **fingerprint layer** (cliffy reactive cells, change
  gating, merge mixing at UI-state granularity).
- Fingerprints derived from these encodings are *structured
  projections*: a downstream system can lift `Digest` word pairs and
  exact blades into richer algebras (amari `Multivector<N,0,0>`, N > 3)
  without re-deriving semantics.
- Higher-grade arbitration (Schubert capability seam, Unit 3) runs in
  amari/Borsalino territory and *consumes* these encodings at its
  boundary; it never pushes its own algebra back into GA3.

## 8. Verification obligations

Every impl ships with tests (in `src/geometric.rs`'s test module or the
type's home module):

1. **Class A:** exact round-trip on representative values and defaults.
2. **Class B:** change-soundness — a representative distinct-value pair
   yields distinct multivectors; digest stability — same value yields
   the identical multivector.
3. **True encodings:** algebraic properties hold — roster sum equals
   manual sum of presence encodings; tone census exact on a mixed
   roster; census invariant under participant reordering (commutativity).
4. **Reader DoD:** the annotation output changes when tone mix changes
   and tracks the multivector, not the typed value (test recomputes via
   the multivector alone).

## 9. Prohibitions

- No `GA3::zero()` impls (a zero encoding is the degenerate "everything
  collided" case and defeats change gating).
- No single-blade raw hashes of structured types (ad-hoc hash garbage).
- No encoding whose nonzero coefficients are not attributable to named
  fields in the register.
- No reconstruction claims for Class B.
