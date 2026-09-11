# Discovery Report — amari & karpal against Knopper

**Date:** 2026-08-22
**Runners:** `amari` 0.24.1 (amari-discovery), `karpal` 0.9.0 (karpal-discovery, built
from workspace `--features lonis`)
**Context:** identity-restoration plan (`docs/plans/2026-08-21-identity-restoration.md`),
Units 2 (GA honesty) and 3 (Schubert seam). Release deferred; nothing pinned.
**Artifacts:** `amari inspect`/`recommend` + `karpal imports` envelopes captured in this
session's temp discovery dir; findings summarized here.

## Why this matters now

The maintainer's positioning: GA is the identity (perf barriers accepted, contended
with); **amari's Cayley tables** already aid the complex GA/Schubert operations that
would otherwise be brute force; the very-long-term roadmap is **optical hardware**
where GA ops are effectively free — the substrate is a hardware bet, not a tax.

## amari-discovery findings

### Inspection (Knopper @ develop b180b60)

Compatible; no direct amari dependencies detected (correct — Knopper's GA today is
cliffy-core's `GA3` fingerprint layer only). Replayable envelope retained.

### Recommend — encoding goal

> "encode UI state types injectively into GA3 multivectors with compositional blade
> assignments; derive presence and collaboration semantics from multivector structure
> for reactive change detection and later Schubert merge arbitration"

**Preferred:** `amari:amari-core:product:geometric-product`
(`amari_core::Multivector::geometric_product`, amari-core 0.24.1, feature `std`),
confidence 0.63, with full integration steps (dep → feature → symbol → example →
probe → test) and a runnable probe `amari-probe:core:geometric-product:v1`.

Read: the catalog's entry point for Unit 2's compositional blade work is the
geometric product itself — tone mixing, roster sums, and later subspace composition
all route through it.

### Catalog vs. codebase gap (important for Unit 3)

The curated catalog (17 capabilities) does **not** yet expose the Schubert/Cayley
machinery, but the codebase has it:

| Machinery | Location | Surface |
| --------- | -------- | ------- |
| Schubert calculus | `amari-enumerative` | `SchubertClass`, `SchubertCalculus::{intersection_number, multi_intersect → IntersectionResult, product, pieri_multiply, lr_cached}`, Giambelli determinant, `FlagVariety` |
| Grassmannian cells | `amari-core::gf2::grassmannian` | `schubert_cell_of(subspace) → partition`, `gaussian_binomial`, `enumerate_subspaces` |
| Cayley tables | `amari-core::cayley` | `CayleyTable<P,Q,R>::get_product(i,j)` — table-driven GA products (optimization pass pending per maintainer) |
| Cayley navigation | `amari-automata` | `CayleyNavigator`, `CayleyGraphNavigator`, `GroupElement::to_multivector::<3,0,0>()` — **GA3-native** |

Implications:

1. **Unit 3's capability seam can consume real machinery now** — `intersection_number`
   / `multi_intersect` are exactly the roadmap-05 merge arbitration; the amari-discovery
   catalog simply hasn't catalogued them yet (an upstream catalog contribution, not a
   blocker).
2. **`schubert_cell_of` is the subspace→condition encoder** — the bridge for
   "capabilities as Schubert conditions" already exists in amari-core.
3. **Cayley tables give the cheap composition path** for blade-level operations in
   Unit 2's encodings (tone mixing as table lookups rather than product expansion) —
   pending the maintainer's optimization pass.
4. **`to_multivector::<3,0,0>` confirms GA3 ↔ group-element alignment** — the
   Cayley-navigation layer speaks Knopper's (cliffy's) exact algebra signature.

## karpal-discovery findings

### Imports

0 resolved symbols (Knopper consumes no karpal crates — honest zero). The value is
the **83-concept mathematical overlay** mapped against Knopper's architecture:

| Knopper concept | Karpal concept | Note |
| --------------- | -------------- | ---- |
| `Machine` (Context + Model + project) | **Store comonad** ("state + focus") + `ComonadEnv` | project ≈ extract/peek at the focus; Context is literally the Env comonad lane |
| projection-injection seam (§3 of the embedding contract) | **Optic** (bidirectional focus) | Model↔Scene bidirectionality is optic-shaped; informs Unit 2's encoding compositionality |
| `Effect<Msg>` (closed enum today; §O3 async bridge open) | **Free monad / Freer** | the open-effects question is free-monad territory; the closed enum is its one-constructor-at-a-time quotient |
| `resolve_layout` (scene tree → rects) | **Recursion schemes** (`cata`/`hylo`) | layout is a catamorphism; hylo fuses measure+resolve phases |
| scene combinator map chains | **Yoneda / Codensity** | fusion opportunity if scene combinators ever chain maps |
| presence tones / merge-joins | **Lattice / HeytingAlgebra / Semiring** | tones as a bounded lattice; merge as join; rates as semiring — candidates for the encoding contract's algebraic vocabulary |
| FRP feedback (cliffy Behaviors) | **Traced monoidal / ArrowLoop** | the reactive loop is traced-monoidal-shaped |

No karpal dependency is proposed for 0.1.0 — this is vocabulary and design-shape
input, especially for the encoding contract's composition laws.

## Feeds into the plan

- **Unit 2**: encoding contract gains (a) geometric product as the composition
  primitive (amari recommend), (b) Cayley-table lookups as the cheap path for blade
  mixing, (c) lattice vocabulary for tones/merges (karpal overlay), (d) optic-shaped
  bidirectionality as the Model↔Scene composition law.
- **Unit 3**: use `amari-enumerative`'s real `SchubertCalculus` + `schubert_cell_of`
  rather than sketching from scratch; contribute the catalog entries upstream so the
  next `amari recommend` run surfaces them.
- **Upstream follow-ups (not blockers)**: amari-discovery catalog entries for
  Schubert calculus / Cayley tables; cliffy-core change-gated notification (Unit 2's
  candidate second reader).
