# Streaming-Append Performance

The embedding contract's §5 requires append-mostly scenes (agent transcripts)
to stay cheap. The Tier 0 spike measured this against 0.1.0 — **it fails
today, and the failure is quantified and scheduled.**

## The measurement

Transcript machine (`Shared = Vec<String>`, stable per-line node ids), 1,000
appends, per-turn `set_shared` + `render_to_backend` against `MockBackend`,
80×24 bounds:

| Appends (avg ms/append) | Debug | Release |
| ----------------------- | ----- | ------- |
| 1–100 | 0.090 | 0.023 |
| 401–500 | 1.630 | 0.241 |
| 900–1000 | 6.062 | 0.778 |

Reproduce: `cargo test --test embedding_harness -- --ignored --nocapture`.

## What's fine vs. what isn't

**Fine:** the committed *patch surface* per append is O(1) — one row insert
plus a header update, bounded backend commands, no visible-window churn.
Stable ids do their job.

**Not fine:** internal per-commit cost grows ~quadratically.

1. `diff_render_ops` is **O(n²)** — a linear scan of the previous op list for
   every op (and again for removals). Dominant at scale.
2. `render_ops(bounds)` re-runs full layout + lowering per turn — O(n).
3. `set_shared` / `sample()` clone the whole `Shared` — O(n) copy, no
   structural sharing.

## The work item (0.1.x)

- HashMap-keyed O(n) `diff_render_ops` (id → index), the single biggest win
- Viewport-bounded projection so machines render the visible window only
- (Later) incremental `Shared` updates instead of whole-projection replace

At 1,000 lines, release-mode commits are sub-millisecond — the first Wallace
slice is unblocked. At 10k lines the current path projects to ~80 ms/append,
so the append-optimized diff lands before long transcripts ship.
