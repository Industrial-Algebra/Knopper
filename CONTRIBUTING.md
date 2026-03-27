# Contributing to Knopper

## Branching and release flow

Knopper follows the Industrial Algebra git workflow:

1. Create a topic branch from `develop` using a typed prefix:
   - `feature/<name>`
   - `fix/<name>`
   - `chore/<name>`
   - `docs/<name>`
   - `refactor/<name>`
   - `test/<name>`
2. Open a pull request into `develop`.
3. Merge validated work into `develop`.
4. Prepare releases with a release pull request from `develop` into `main`.
5. Tag releases from `main`.

## Local quality gates

Commits are expected to pass the same checks as CI:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`

A repository-local git hook is provided in `.githooks/pre-commit`.

Enable it once per clone with:

```bash
./scripts/setup-hooks.sh
```

## Rust engineering guidelines

All contributions should follow these IA Rust conventions:

1. Prefer idiomatic, explicit Rust over clever abstractions.
2. Use a test-driven development workflow whenever practical:
   - write or update tests first,
   - implement the smallest change that satisfies them,
   - refactor while keeping tests green.
3. Use algebraic modelling and phantom types when they improve correctness and encode invariants.
4. Reach for `rayon` when a workload is CPU-bound and parallelizable.
5. Keep public APIs small, composable, and well-tested.
6. Treat clippy warnings as actionable by default.

## Pull request checklist

- [ ] Branch targets `develop`
- [ ] Tests added or updated first
- [ ] `cargo fmt --all` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test --all-features` passes
- [ ] Documentation updated where behavior or workflow changed
