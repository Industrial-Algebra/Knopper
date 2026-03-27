# Knopper Architecture: Collaboration Model

## Status

Draft.

## Purpose

This document defines the first-pass collaboration model for Knopper.

Collaboration is not an optional add-on in Knopper's architecture. It is a first-class design target and one of the framework's primary differentiators.

Knopper should support multi-user TUIs by synchronizing **semantic shared state** and projecting it locally, rather than by sharing terminal byte streams or PTY output.

---

## Core principle

Knopper collaboration is:

> multiple participants operating within a shared reactive state space, each rendering a local terminal projection of convergent shared state plus local ephemeral state.

This is fundamentally different from:
- terminal mirroring,
- screen sharing,
- multiplexed PTY streaming.

---

## Why semantic collaboration

Semantic collaboration provides major advantages:

- each participant can render to a different terminal size,
- theming may remain local,
- local ephemeral focus can differ from shared semantic state,
- collaboration works at the application level,
- diffing and overlays can attach to stable scene identities,
- concurrent edits can converge via CRDT semantics.

This is the appropriate model for collaborative coding tools, AI agent workspaces, and distributed operator consoles.

---

## Foundation libraries

Knopper should use `cliffy-protocols` as its primary synchronization substrate.

Relevant capabilities:
- geometric CRDTs,
- vector clocks,
- delta synchronization,
- lattice-style merge semantics,
- convergence under concurrent updates.

Knopper should use these to synchronize shared machine/application state, not renderer state.

---

## Three-state model

Knopper collaboration depends on a strict separation of state categories.

### 1. Local ephemeral state
Never replicated by default.

Examples:
- local cursor blink,
- viewport cache,
- local hover/focus,
- local terminal measurement cache,
- local theme,
- in-progress IME state.

### 2. Shared replicated state
Replicated and convergent.

Examples:
- documents,
- task boards,
- shared layouts,
- synchronized command history,
- agent orchestration state,
- shared annotations,
- remote selections and semantic cursors.

### 3. Derived collaboration state
Computed from local + shared sources.

Examples:
- presence bars,
- remote cursor overlays,
- sync status badges,
- conflict indicators,
- participant lists,
- shared-vs-local divergence markers.

This separation should be explicit in the machine/runtime model.

---

## Participants and presence

Knopper should model participants semantically.

Likely participant attributes:
- participant ID,
- display name,
- role or capability set,
- color/style identity,
- presence status,
- optional machine/session metadata,
- optional agent/human classification.

This is especially important for collaborative AI tooling, where participants may include:
- humans,
- autonomous agents,
- assistant agents,
- observer or approval roles.

---

## Shared scene anchors

To support collaboration overlays, scene nodes must be addressable.

Collaboration should attach to stable semantic anchors such as:
- document region IDs,
- node IDs,
- machine instance IDs,
- address paths,
- selection/cursor anchors.

Examples:
- remote participant cursor attached to an editor node,
- shared selection attached to a list range,
- sync issue attached to a replicated panel,
- agent presence attached to a task surface.

---

## What should be shared

Knopper should make sharing explicit.

Recommended shareable categories:
- structured domain state,
- scene-relevant semantic state,
- collaborative cursors/selections,
- shared machine/workspace configuration,
- replicated command/task artifacts.

Recommended non-shared-by-default categories:
- renderer surfaces,
- terminal dimensions,
- local style theme,
- local focus ring,
- local animation progress,
- ephemeral measurement caches.

---

## Machine implications

A Machine should explicitly separate:
- local `Model`,
- shared `Shared`,
- and derived collaboration-aware projection.

Machines should be able to:
- observe shared state reactively,
- emit updates that target shared replicated state,
- project collaboration overlays in `Behavior<Scene<Msg>>`.

This implies collaboration is not bolted onto scenes after the fact; it participates in the projection model.

---

## Collaboration overlays

Knopper should support collaboration-aware scene features from early design stages, even if some are implemented later.

Important collaboration overlays include:
- remote cursor indicators,
- remote selection highlights,
- participant layers,
- sync status indicators,
- conflict or divergence markers,
- collaboration-aware viewport markers.

These may appear as:
- dedicated scene nodes,
- annotations consumed during projection/layout,
- or specialized overlay passes.

---

## Sync lifecycle

A first-pass collaboration lifecycle likely includes:

1. local mutation intent or incoming remote delta,
2. application to shared replicated state,
3. convergence/merge,
4. reactive propagation into shared `Behavior<T>`,
5. scene reprojection,
6. collaboration annotation/layout pass,
7. local render.

This allows collaboration to remain semantic and reactive.

---

## Conflict model

Knopper should prefer convergence by construction where possible through `cliffy-protocols`.

However, convergence does not eliminate all UX-level conflict concerns.

There may still be a need to visualize:
- concurrent edits,
- divergence windows,
- stale participant views,
- pending sync state,
- semantic overwrite concerns.

Therefore Knopper should separate:
- **data convergence**, and
- **user-visible conflict presentation**.

A convergent CRDT merge may still warrant UI indication.

---

## Local rendering of shared state

Each participant should render the same shared semantics through local runtime constraints.

This means:
- terminal size can differ,
- clipping/viewport can differ,
- theme can differ,
- some focus state can remain local,
- overlays may be personalized,
- but the shared state basis remains convergent.

This is a major advantage over shared terminal-stream approaches.

---

## AI agent suite relevance

Knopper's collaboration model is especially powerful for AI coding and orchestration environments.

Potential collaborative agent scenarios:
- shared code-navigation surfaces,
- agent-human review panels,
- remote cursor/selection in a TUI editor,
- synchronized task decomposition boards,
- shared command/control for multiple agents,
- replicated logs/traces with semantic annotations,
- collaborative terminal dashboards with agent status surfaces.

Knopper should treat human and agent participants as first-class citizens in the collaboration model.

---

## Security and authority considerations

A later ADR should address authority boundaries, but the collaboration model should anticipate them.

Important future concerns:
- read-only vs write-capable participants,
- approval workflows,
- per-node or per-surface permissions,
- trusted vs untrusted agent participants,
- auditability of shared actions.

This is particularly relevant for coding-agent applications.

---

## First implementation recommendation

Initial implementation should prove the collaboration model with a small semantic example rather than a full collaborative editor.

Good first targets:
- shared counter or list state,
- shared task panel,
- remote participant presence strip,
- sync status surface,
- remote selection marker in a simple list.

This validates the architecture without needing full distributed editing immediately.

---

## Open follow-up questions

1. exact Knopper abstraction over `cliffy-protocols`
2. participant/presence type design
3. shared anchor addressing scheme
4. permissions and authority model
5. whether collaboration is feature-gated at crate or node level
6. delta transport abstraction and integration points

These should be addressed in later ADRs.
