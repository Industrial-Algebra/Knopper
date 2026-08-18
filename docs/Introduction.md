# Knopper

Knopper is a Rust framework for building functional-reactive terminal user
interfaces with a machine-centered programming model, a terminal-native
scene algebra, and a rendering pipeline tailored to high-performance
Notcurses backends. It is designed multi-user from the ground up as a
collaboration substrate for downstream Industrial Algebra projects.

## Reading map

- **New to Knopper?** Start with [Writing a machine](./guides/01-writing-a-machine.md),
  then [Composing machines](./guides/02-composing-machines.md).
- **Want the design?** The [Architecture](./architecture/00-first-pass-architecture.md)
  chapters cover the machine model, scene algebra, runtime pipeline, and
  rendering model.
- **Building multi-user?** Read the
  [Collaboration-ready contract](./roadmap/06-collaboration-ready-contract.md)
  first — it is the canonical statement of the embedding seam downstream
  projects program against.
- **Shipping?** See the [release preparation](./release-0.1.0.md) notes.

## Status

Knopper is pre-1.0. The `0.1.0` API is coherent and tested but unstable;
expect refinement in `0.2.0`, especially around the collaboration seam and
demo organization. See [CHANGELOG](https://github.com/Industrial-Algebra/Knopper/blob/develop/CHANGELOG.md)
for what shipped.

## Repository layout

| Path | Contents |
| ---- | -------- |
| `src/` | Library core: machine, scene, runtime, focus, layout, render, backends |
| `src/standard/` | Standard machines (list, input, tabs, palette, modal, …) |
| `src/collaboration.rs` | Canonical `Shared` payload types for multi-user sessions |
| `tests/` | Integration tests against the mock backend |
| `docs/architecture/` | Design chapters (this book's Architecture section) |
| `docs/guides/` | Task-oriented guides |
| `docs/roadmap/` | Milestone specs and design contracts |
