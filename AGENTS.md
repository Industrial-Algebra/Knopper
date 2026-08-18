# Knopper — Agent Operating Guide

## Gitflow (Non-Negotiable)

Knopper follows IA gitflow as defined in the
[`ia-gitflow`](https://github.com/Industrial-Algebra/ia-toolkit/blob/main/skills/ia-gitflow/SKILL.md)
skill. Read it before touching branches.

### Branch Model

```
feature/* ──PR──▶ develop ──release PR──▶ main ──tag v*──▶ publish
                     ▲                                        │
                     └──────── backmerge (merge commit) ──────┘
```

### Hard Rules

1. **Never push directly to `main` or `develop`.** Both are protected.
   No direct pushes — not "just a CI fix", not "a one-liner", not "it's faster".
   Branch it, PR it, let CI run. This is enforced by GitHub branch protection.

2. **Every release to `main` is followed by a `main → develop` backmerge**
   using a merge commit (never squash). This is the last step of releasing,
   not an optional chore.

3. **Release-only commits (version bump, changelog dating) live on a
   `release/*` branch**, not on `develop` or `main`.

### Branch Protection

Both `main` and `develop` have:
- Required status checks (Format, Clippy, Test, Documentation)
- `allow_force_pushes: false`
- `allow_deletions: false`

### CI Layout

- **Format / Clippy / Test / Documentation** run on GitHub-hosted runners
  on the **stable** toolchain with default features.
- **Notcurses (self-hosted)** runs `--features notcurses` clippy/test/doc on
  the `knopper-notcurses-x64-01` runner (system notcurses 3.0.17; Ubuntu
  24.04 hosted images ship 3.0.7, too old for libnotcurses-sys). Gated:
  runs on pushes to `main`, PRs to `main`, and PRs labeled `run-notcurses`.
- **Do not** add `--all-features` to hosted jobs — it pulls in notcurses
  and fails on the missing/too-old system library.

### Release Checklist

1. Feature PRs merged to `develop`
2. Version bump + changelog dating on a `release/*` branch off `develop`
3. Release PR: `release/*` → `main` (Notcurses self-hosted job runs here)
4. User reviews and merges
5. Tag `v*` on main → triggers publish workflow
   (requires the `CARGO_REGISTRY_TOKEN` repo secret)
6. **Backmerge** `main → develop` (merge commit)
7. Announce via `ia-website` skill

## Coding Standards

Follow the
[`ia-coding-standards`](https://github.com/Industrial-Algebra/ia-toolkit/blob/main/skills/ia-coding-standards/SKILL.md)
skill: TDD (test first), phantom types, `Result` not panic, exhaustive matching,
feature gates additive only, every public item documented.

## Collaboration Contract

Multi-user support is a first-class design constraint. The canonical
`Shared` payload types live in `src/collaboration.rs`; the embedding and
Schubert capability seams are specified in
`docs/roadmap/06-collaboration-ready-contract.md`. Focus state is
participant-local — do not route it through `Shared`.

## License

Apache-2.0. See `LICENSE` and `CONTRIBUTING.md`.
