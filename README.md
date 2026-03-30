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

Run the current demo snapshot binary with:

```bash
cargo run
```

The current binary prints a rendered snapshot of a composed Knopper demo workspace built from the standard machines.

## Contributor guidance

See [CONTRIBUTING.md](CONTRIBUTING.md) for workflow and coding guidelines.

## Guides

- [docs/guides/01-writing-a-machine.md](docs/guides/01-writing-a-machine.md)
- [docs/guides/02-composing-machines.md](docs/guides/02-composing-machines.md)
- [docs/guides/03-focus-and-modal-semantics.md](docs/guides/03-focus-and-modal-semantics.md)
- [docs/guides/04-scene-layout-render-pipeline.md](docs/guides/04-scene-layout-render-pipeline.md)
- [docs/guides/05-walkthrough-demo-workspace.md](docs/guides/05-walkthrough-demo-workspace.md)

## Architecture

Architecture drafts:

- [docs/architecture/00-first-pass-architecture.md](docs/architecture/00-first-pass-architecture.md)
- [docs/architecture/01-machine-model.md](docs/architecture/01-machine-model.md)
- [docs/architecture/02-scene-algebra.md](docs/architecture/02-scene-algebra.md)
- [docs/architecture/03-collaboration-model.md](docs/architecture/03-collaboration-model.md)
- [docs/architecture/04-runtime-pipeline.md](docs/architecture/04-runtime-pipeline.md)
- [docs/architecture/05-rendering-model.md](docs/architecture/05-rendering-model.md)

## Roadmap

- [docs/roadmap/00-first-release-roadmap.md](docs/roadmap/00-first-release-roadmap.md)
- [docs/roadmap/01-collaboration-readiness.md](docs/roadmap/01-collaboration-readiness.md)
- [docs/roadmap/02-standard-machine-collaboration-audit.md](docs/roadmap/02-standard-machine-collaboration-audit.md)
- [docs/roadmap/03-participant-local-presence-overlays.md](docs/roadmap/03-participant-local-presence-overlays.md)
- [docs/roadmap/04-textarea-editor-design.md](docs/roadmap/04-textarea-editor-design.md)

## Engineering baseline

This repository follows IA engineering standards:

- topic branch -> PR to `develop` -> release PR -> `main`
- local git hooks for `fmt`, `clippy`, and `test`
- CI for `fmt`, `clippy`, and `test` on pushes and pull requests
- idiomatic Rust with TDD-first expectations
- algebraic modelling with phantom types where useful
- `rayon` for CPU parallelism when appropriate
