# Embedding Contract (Wallace)

The Wallace ↔ Knopper embedding contract lives in
`IA-documents/CONTRACTS/wallace-knopper-embedding.md` (v0.4). It defines the
boundary every host builds against. **It was validated against the 0.1.0 code
on 2026-08-18** (the Tier 0 spike); this chapter summarizes the validated
state. The findings note with full evidence is `docs/embedding-validation-2026-08.md`
in the repo.

## Verdicts

| § | Assumption | Status |
| - | ---------- | ------ |
| §1 | Embedder-driven `Runtime`; no mandatory `run()` | ✅ Verified |
| §2 | Host-driven turn cycle | ✅ Verified end-to-end |
| §3 | `set_shared` reprojects; `Model` participant-local | ✅ Verified, **signed off against the collaboration contract — they agree** |
| §4 | Host-defined `Msg`; effect semantics | ✅ Verified, with one correction (below) |
| §5 | Cheap append-mostly diff | ❌ Quadratic today — measured, scheduled for 0.1.x |

## The §4 correction

The contract originally said the host's adapter "interprets effects requiring
orchestration." In the shipped code, `Effect` is a **closed enum**
(`None` / `Emit` / `Batch` / `RequestFocus`) **drained synchronously** by the
runtime — no effect reaches the host. Orchestration is host-triggered at
send-time (the adapter saw the domain event first) or by observing `Model`.
The contract's v0.4 reflects this; a host-orchestration effect variant remains
an open question (§O3).

## The §3 sign-off

The one question that could have split downstream consumers — does
participant-local UI state live in `Model` (embedding contract §3/§7) or
somewhere the collaboration module owns — is settled: **both contracts put
focus/scroll/selection in participant-local `Model`, never in `Shared`**, and
the harness proves two runtimes sharing a projection diverge without
cross-talk. Wallace, Tsume, and Dominic can pin against either document.

## The harness

`tests/embedding_harness.rs` is the canonical example: host-defined domain
events and `Msg`, a hash-encoded projection type, the five-turn drive loop,
participant-locality assertions, and the streaming-append measurement.
