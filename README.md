# Knopper

Knopper is an Industrial Algebra Rust project.

## Engineering baseline

This repository is set up to follow IA standards:

- gitflow-like delivery: topic branch -> PR to `develop` -> release PR -> `main`
- local git hooks for `fmt`, `clippy`, and `test`
- CI for `fmt`, `clippy`, and `test` on pushes and pull requests
- idiomatic Rust with TDD-first expectations
- algebraic modelling with phantom types where useful
- `rayon` for CPU parallelism when appropriate
- `rayon` for CPU parallelism when appropriate

## Getting started

Enable repository git hooks after cloning:

```bash
./scripts/setup-hooks.sh
```

## Contributor guidance

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full workflow and coding guidelines.

## Architecture

Architecture drafts:

- [docs/architecture/00-first-pass-architecture.md](docs/architecture/00-first-pass-architecture.md)
- [docs/architecture/01-machine-model.md](docs/architecture/01-machine-model.md)
- [docs/architecture/02-scene-algebra.md](docs/architecture/02-scene-algebra.md)
- [docs/architecture/03-collaboration-model.md](docs/architecture/03-collaboration-model.md)
- [docs/architecture/04-runtime-pipeline.md](docs/architecture/04-runtime-pipeline.md)
- [docs/architecture/05-rendering-model.md](docs/architecture/05-rendering-model.md)
