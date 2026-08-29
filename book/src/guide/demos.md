# Running the Demos

Knopper ships two reference applications as its binary targets. **They are
reference material, not stable API** — expect them to move or be feature-gated
in 0.2.0.

## The workspace host

```bash
cargo run                              # raw-key interactive host
cargo run --features notcurses -- --notcurses
```

An interactive workspace exercising the standard machines: list, input,
tabs, command palette (Ctrl+K or `/`), modal confirm — with declarative focus
traversal and palette trapping. The raw host and the notcurses host drive the
same `Runtime` through different backends, which is the embedding story in
miniature.

## The review workspace

```bash
cargo run -- --demo review
```

A review-focused composition: query input, draft editing, list-detail
navigation — the shape of a code-review control panel.

## What to look at in the source

- `src/main.rs` — three host loops (`run_shell`, `run_raw_host`,
  `run_review_raw_host`): terminal lifecycle, key translation, the turn cycle
- `src/demo.rs`, `src/demo_ui.rs` — the workspace machine and its scene
- `src/review_demo.rs` — the review machine

Presence cues in the demos are **cosmetic placeholders** — they demonstrate
where `PresenceSlot`/`RemoteCursor` annotations attach, driven by hardcoded
data rather than a live `ParticipantRoster`. Deriving them from a real roster
in `Shared` is a 0.2.0 item.

## Headless

Everything the demos do runs against `MockBackend` in tests — see
`tests/runtime_pipeline.rs` and `tests/embedding_harness.rs` for host-free
driving patterns.
