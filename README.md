# Knopper

Knopper is a Rust framework for building **functional-reactive terminal user interfaces** with a **machine-centered programming model**, a **terminal-native scene algebra**, and a rendering pipeline tailored toward **high-performance Notcurses backends**.

It is being designed as both:

- a reusable TUI framework for composing rich terminal applications, and
- a collaboration-ready UI substrate for downstream Industrial Algebra projects that need multi-participant semantic interfaces rather than terminal mirroring.

## Project goals

Knopper aims to provide:

- a public abstraction centered on **`Machine`**
- semantic UI projection as **`Behavior<Scene<Msg>>`**
- a compositional scene/layout/render/runtime pipeline
- reusable standard machines for common terminal controls
- explicit focus, modal, and overlay semantics
- backend isolation with strong support for **Notcurses**
- architectural seams for future **collaborative/shared-state runtimes**

Knopper is intentionally **not** a DOM clone, JSX layer, or shared-terminal-stream system. Its model is centered on semantic state, local projection, and terminal-native interaction.

## Current status

Knopper is in an **early but substantial framework stage**.

Today the project already includes:

- core `Machine` and `PureMachine` abstractions
- scene algebra with layout, overlay, scroll, alignment, annotation, and focus-scope support
- runtime event routing, update/effect application, reprojection, layout, rendering, diffing, and backend command generation
- mock renderer/backend test infrastructure
- a feature-gated **Notcurses** backend
- standard machines including:
  - list
  - input
  - button
  - toggle/checkbox
  - tabs / segmented selector
  - textarea
  - list-detail composition
  - command palette
  - reusable modal helper
- scene-derived, scope-aware focus traversal with runtime `Tab` / `Shift-Tab` handling

## Near-term release focus

The current release effort is focused on reaching a coherent `0.1.0` experimental release with:

- a stable-enough machine-first core API
- a small but useful standard-machine set
- principled focus and modal interaction semantics
- reliable Notcurses-backed rendering for real demos
- collaboration-ready architectural boundaries for downstream IA projects

## Getting started

Enable repository git hooks after cloning:

```bash
./scripts/setup-hooks.sh
```

Run the quality gate locally with:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

Run the current raw-key interactive demo host with:

```bash
cargo run
```

Optional Notcurses-backed rendering during the raw host:

```bash
cargo run --features notcurses -- --notcurses
```

If you want the older command-driven shell instead, use:

```bash
cargo run -- --shell
```

Note: `--shell` is a diagnostic command driver, not a fullscreen TUI. It prints the command list plus a textual snapshot/debug view of the demo state.

The raw-key host is the actual interactive terminal demo. It renders a composed Knopper workspace built from the standard machines and supports direct raw-key interaction, including focus traversal, textarea editing, list navigation, quick-actions palette activation via `Ctrl-P`, and a toggleable inspector via `Ctrl-G` or `F2`.

The project now includes a second review-focused demo accessible from the main host as well:

```bash
cargo run -- --demo review
cargo run --features notcurses -- --notcurses --demo review
```

Cargo's default run target is now the main `Knopper` host binary, so plain `cargo run ...` launches that host. The older standalone `review_demo` binary still exists, but the preferred path is to use the main host with `--demo review`.

This review-focused workspace exists specifically to exercise the extracted composition helper layer in `src/demo_ui.rs` against a second application shape.

The current demo workspace is intentionally a little more app-like than a bare control gallery:

- tabbed workspace header
- shared-mode toggle and sync action row
- sectioned Notes and Tasks surfaces
- participant-local collaboration cues in notes/tasks detail areas
- quick-actions modal for common workspace commands
- derived status footer summarizing the current workspace state

It also now serves as a stronger composition template: the demo uses reusable scene-building helpers for framed surfaces, presence strips, and labeled detail rows so downstream applications can follow the same parent-machine composition style.

## Contributor guidance

See [CONTRIBUTING.md](CONTRIBUTING.md) for workflow and coding guidelines.

## Guides

- [docs/guides/01-writing-a-machine.md](docs/guides/01-writing-a-machine.md)
- [docs/guides/02-composing-machines.md](docs/guides/02-composing-machines.md)
- [docs/guides/03-focus-and-modal-semantics.md](docs/guides/03-focus-and-modal-semantics.md)
- [docs/guides/04-scene-layout-render-pipeline.md](docs/guides/04-scene-layout-render-pipeline.md)
- [docs/guides/05-walkthrough-demo-workspace.md](docs/guides/05-walkthrough-demo-workspace.md)
- [docs/guides/06-running-the-interactive-demo.md](docs/guides/06-running-the-interactive-demo.md)
- [docs/guides/07-reusable-composition-patterns.md](docs/guides/07-reusable-composition-patterns.md)
- [docs/guides/08-runtime-event-policy.md](docs/guides/08-runtime-event-policy.md)

## Architecture

Architecture drafts:

- [docs/architecture/00-first-pass-architecture.md](docs/architecture/00-first-pass-architecture.md)
- [docs/architecture/01-machine-model.md](docs/architecture/01-machine-model.md)
- [docs/architecture/02-scene-algebra.md](docs/architecture/02-scene-algebra.md)
- [docs/architecture/03-collaboration-model.md](docs/architecture/03-collaboration-model.md)
- [docs/architecture/04-runtime-pipeline.md](docs/architecture/04-runtime-pipeline.md)
- [docs/architecture/05-rendering-model.md](docs/architecture/05-rendering-model.md)
- [docs/architecture/06-application-layout-patterns.md](docs/architecture/06-application-layout-patterns.md)

## Roadmap

- [docs/roadmap/00-first-release-roadmap.md](docs/roadmap/00-first-release-roadmap.md)
- [docs/roadmap/01-collaboration-readiness.md](docs/roadmap/01-collaboration-readiness.md)
- [docs/roadmap/02-standard-machine-collaboration-audit.md](docs/roadmap/02-standard-machine-collaboration-audit.md)
- [docs/roadmap/03-participant-local-presence-overlays.md](docs/roadmap/03-participant-local-presence-overlays.md)
- [docs/roadmap/04-textarea-editor-design.md](docs/roadmap/04-textarea-editor-design.md)
- [docs/roadmap/05-distributed-correctness-schubert.md](docs/roadmap/05-distributed-correctness-schubert.md)
- [docs/roadmap/06-collaboration-ready-contract.md](docs/roadmap/06-collaboration-ready-contract.md)

## Engineering baseline

This repository follows IA engineering standards:

- topic branch -> PR to `develop` -> release PR -> `main`
- local git hooks for `fmt`, `clippy`, and `test`
- CI for `fmt`, `clippy`, and `test` on pushes and pull requests
- idiomatic Rust with TDD-first expectations
- algebraic modelling with phantom types where useful
- `rayon` for CPU parallelism when appropriate
