# Embedding-Contract Validation — Findings (2026-08-18)

**Scope:** Tier 0 spike (`docs/handoff/2026-08-17-tier0-embedding-validation.md`).
**Contract:** `IA-documents/CONTRACTS/wallace-knopper-embedding.md` v0.3, §1–§5.
**Code under test:** Knopper develop @ `45ed84c` (post PR #5 + #6).
**Harness:** `tests/embedding_harness.rs` — 6 tests + 1 ignored perf measurement,
all against the public `Runtime` surface with host-defined types only.

## Verdicts per assumption

| Contract section | Assumption | Verdict |
| --- | --- | --- |
| §1 | `Runtime` surface is embedder-driven (all listed methods present) | ✅ Verified |
| §1 | Non-goal: no mandatory `run()` loop in the library | ✅ Holds (only the demo binary loops) |
| §2 | Host-driven turn cycle: dispatch / send / set_shared (+set_context) / diff / commit | ✅ Verified end-to-end |
| §3.1 | `set_shared` reprojects scenes that read `Shared` | ✅ Verified (Behavior reactivity; lib test `runtime_shared_state_updates_reproject_scene` survives M2) |
| §3.2 | `Model` = participant-local; agrees with roadmap 06 | ✅ **Signed off — the two contracts agree** |
| §3.3 | `set_shared` whole-replace composes with machine `update` on `Model` | ✅ Verified |
| §4 | Host defines its own `Msg` without Knopper internals | ✅ Verified |
| §4 | Effects "interpreted by the adapter" | ⚠️ **Correction needed** — see below |
| §5 | `diff()` cheap for append-mostly scenes | ❌ **Fails today** — quadratic; 0.1.x work item |

## §1 — Embedding model

Every method the contract lists exists with the expected shape:
`new(machine, ctx, shared_initial)`, `set_context`, `set_shared`,
`dispatch(RuntimeEvent)`, `send(Msg)`, `scene`, `model`, `shared`, `focus`,
`layout(bounds)`, `render_ops(bounds)`, `cursor(bounds)`, `diff(bounds)`,
`render_to_backend{,_with_cursor,_auto_cursor}`, `invalidate_render_state`.
Plus `render(renderer, bounds)` for the generic `Renderer` path.

The library has **no `run()`**. The only event loops live in the demo binary
(`src/main.rs`), which drives `Runtime` exactly as a host would. The PR #5
runtime event policy (`docs/guides/08-runtime-event-policy.md`) is a dispatch
*precedence* policy — it introduced no hosted/blocking loop. Non-goal intact.

## §2 — Drive loop

`s2_host_driven_turn_cycle` runs the contract's loop verbatim over five turns:
domain event → adapter → `send` / `set_shared`; key → `dispatch`; commit via
`render_to_backend`; per-turn `cursor` read. No event-loop ownership on
Knopper's side. One semantic worth host attention: **`diff(bounds)` is
read-only** — it diffs against `last_render_ops` but does not advance the
baseline. The committing paths (`render`, `render_to_backend*`) advance it.
A host that calls `diff()` repeatedly without committing sees cumulative
patches, not incremental ones.

## §3 — Projection-injection (the spike's center)

1. **Reprojection on `set_shared`: confirmed.** `Runtime::set_shared` sets the
   shared `Behavior`; the scene behavior re-derives on the next sample. The
   pre-existing lib test survived the M2 collaboration rework and still passes.
2. **`Model` participant-local: SIGNED OFF.** Roadmap 06
   (`docs/roadmap/06-collaboration-ready-contract.md`) states "Focus is
   participant-local. Each runtime's `FocusState` is its own" and lists
   selection/commit as `Model`-resident — identical to contract §3/§7.
   `s3_model_state_is_participant_local` proves two runtimes sharing a pushed
   projection diverge in `Model`/focus without cross-talk. **No disagreement
   between the contracts; no reconciliation needed before 0.1.0.**
3. **Whole-replace composes with `update`: confirmed.** `send()` mutates
   `Model`; `set_shared` replaces `Shared`; the next projection reflects both
   (`s3_set_shared_reprojects_and_composes_with_update`). "Works," not
   "optimal" — see §5.

**New API fact the contract should record:** hosts with custom `Model` /
`Shared` types must implement cliffy-core's `IntoGeometric` / `FromGeometric`
(the `Behavior` reactivity fingerprint). These are now **re-exported from
`knopper`** (`IntoGeometric`, `FromGeometric`, `GA3`) so hosts need no direct
cliffy-core dependency. Encodings may be hash-based (the native value is
cached; the geometric form is only a fingerprint), exactly as cliffy-core's
own `String` impl does.

## §4 — Message + effect contract

Host-defined `Msg` on a `PureMachine`: verified, no Knopper internals needed.

**Correction needed (contract §4, third bullet).** The contract says Wallace's
adapter "interprets effects requiring orchestration." The actual `Effect<Msg>`
enum is **closed**: `None`, `Emit(Msg)`, `Batch(Vec<Effect>)`,
`RequestFocus(NodeId)`. The runtime drains effects **synchronously and
completely** inside `send`/`dispatch` (`Emit` recursively self-dispatches;
`Batch` applies in order; `RequestFocus` routes through the focus dispatch
path). No effect reaches the host; there is nothing for an adapter to
interpret today. The working pattern for orchestration ("start a run",
"invoke a tool"): the host triggers async work **at `send`-time in its own
adapter** (it sees the domain event before Knopper does) or observes `Model`
state between turns. §O3 (async-effect bridge) stays open and should be
reworded to note the closed-enum status quo.

## §5 — Streaming-append measurement

Method: transcript machine (`Shared = Vec<String>` projection, stable per-line
node ids), 1,000 appends, per-turn `set_shared` + `render_to_backend` against
`MockBackend`, bounds 80×24.

**Structural assertion (CI-stable):** per-append backend command count stays
bounded (≤ 4: one row insert + header update; `s5_append_patches_stay_bounded`).
The *committed patch surface* per append is O(1) — correct behavior.

**Wall-clock (the number the contract asked for):**

| Appends (avg ms/append) | Debug | Release |
| --- | --- | --- |
| 1–100 | 0.090 | 0.023 |
| 401–500 | 1.630 | 0.241 |
| 900–1000 | 6.062 | 0.778 |
| Max single append | 6.664 | 0.861 |

**Verdict: §5 fails today.** Cost grows ~quadratically (~34–67× for 10× data).
Root causes, in dominance order:

1. `diff_render_ops` is **O(n²)** — linear `find` over the previous op list
   per next op (and again for removals). Dominates at scale.
2. `render_ops(bounds)` re-runs full layout + render lowering per turn — O(n)
   per chunk (the contract's anticipated "re-walk").
3. `set_shared` + `sample()` clone the whole `Shared` — O(n) memcpy of string
   bytes per append (Behavior caches native values; no structural sharing).

At 1,000 lines release-mode commits stay sub-millisecond — usable for the
first slice — but 10k-line transcripts project to ~80 ms/append. **0.1.x work
item (not a contract change, per the contract's own anticipation):** an
append-optimized path — HashMap-keyed O(n) `diff_render_ops`, then
viewport-bounded projection so machines render the visible window only.

## Contract corrections to land in IA-documents (§9 protocol)

1. §4 bullet 3 → reflect the closed `Effect` enum + synchronous drain;
   reword §O3 accordingly.
2. §3 → note the `IntoGeometric` / `FromGeometric` requirement for custom
   `Model`/`Shared`, re-exported from `knopper`.
3. §5 status line → "validated with findings": quadratic confirmed, numbers
   above, append-optimized diff is a scheduled 0.1.x work item.
4. Change log → v0.4 entry.

## Definition of done

- ✅ Harness merged (`tests/embedding_harness.rs`), CI-green.
- ✅ Streaming-append cost is a known number, not a guess.
- ✅ `Model`/`Shared` question signed off (contracts agree).
- ⏳ Contract v0.4 corrections land via IA-documents PR (with this note
  referenced).
