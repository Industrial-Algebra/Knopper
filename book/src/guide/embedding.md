# Embedding Knopper

Knopper is a library substrate: **your process, your event loop, your
terminal lifecycle.** This is contractual — the Wallace ↔ Knopper embedding
contract (IA-documents, validated 2026-08-18 against the 0.1.0 code) defines
this boundary.

## What the host owns

- The async runtime and event loop (tokio, threads, whatever)
- stdin / resize / raw-mode lifecycle, signals, exit
- The domain: sessions, participants, runs, artifacts
- The collaboration transport and convergence

## What Knopper provides

A synchronous, embedder-driven `Runtime<M>`. There is **no `run()` loop** in
the library — the only loops are in the demo binary, which drives `Runtime`
exactly as your host would.

## The turn cycle

```rust
loop {
    match host_event().await {
        HostEvent::Key(k)   => runtime.dispatch(translate_key(k)),
        HostEvent::Resize(r)=> runtime.dispatch(RuntimeEvent::Resize(r)),
        HostEvent::Domain(e)=> {
            let (msg, projection_changed) = adapter.translate(e);
            if let Some(m) = msg { runtime.send(m); }
            if projection_changed { runtime.set_shared(adapter.projection()); }
        }
    }
    runtime.render_to_backend(&mut backend, bounds)?;   // commit
    let cursor = runtime.cursor(bounds);                // place cursor
}
```

Synchronous UI turns; all async I/O stays host-side and surfaces as later
`send` / `set_shared`.

## Contract facts the host must know

- **`set_shared` re-projects.** Machines whose `project` reads `Shared` see
  the new projection on the next sample — no manual invalidation.
- **`set_shared` whole-replaces.** It composes fine with machine `update`
  (they touch different lanes), but granularity is whole-projection; see
  [Streaming-Append Performance](../design/streaming-append.md) for cost.
- **Effects don't escape.** `Effect` is a closed enum drained inside
  `send`/`dispatch`. Orchestration ("start a run") is host-triggered at
  send-time — the adapter saw the domain event first — or by observing
  `Model` between turns.
- **Custom `Model`/`Shared` types** implement `IntoGeometric`/`FromGeometric`,
  re-exported from `knopper`. No direct cliffy-core dependency needed.
- **`diff()` is read-only**; `render_to_backend*` commits and advances the
  baseline.

## Reference harness

`tests/embedding_harness.rs` is the contract-validation harness: a complete
host with domain events, adapter translation, projection pushes, and
multi-runtime participant-locality assertions. Copy it.
