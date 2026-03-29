# Knopper first-release roadmap

## Purpose

This roadmap turns the current architecture and implementation analysis into an execution plan for reaching a usable first release of Knopper.

The intent is to keep the project focused on shipping a coherent, terminal-native, machine-based TUI framework rather than diffusing effort across too many speculative features at once.

At the same time, the roadmap treats collaboration as a first-class design constraint at every stage. Knopper does not need a complete collaboration runtime in `0.1.0`, but `0.1.0` should avoid painting the architecture into a single-user corner because collaborative UI is one of the project’s central differentiators and a dependency concern for downstream IA projects.

## Current position

Knopper has moved beyond a pure architecture draft and now has a real end-to-end framework skeleton:

- `Machine` as the primary public abstraction
- semantic projection as `Behavior<Scene<Msg>>`
- a scene algebra with structural and interaction-oriented wrappers
- runtime event routing, update, reprojection, layout, render lowering, diffing, and backend command generation
- mock renderer/backend test infrastructure
- a real Notcurses backend behind a feature flag
- reusable standard machines:
  - `ListMachine`
  - `InputMachine`
  - `ListDetailMachine`
  - `CommandPaletteMachine`
  - reusable modal helper
- focus foundations:
  - `FocusState`
  - `FocusPath`
  - `FocusOrder`
  - named focus scopes
  - nearest-scope derivation
  - runtime `Tab` / `Shift-Tab` traversal

This is a strong early-framework state. The core question is no longer whether the architecture can work, but how to refine semantics and grow the reusable machine ecosystem toward a credible first release.

## First-release goal

Knopper `0.1.0` should be a usable experimental framework release for building small but real terminal applications with a coherent programming model.

That first release does **not** need to solve every long-term goal. It should provide:

- a coherent single-user application framework
- explicit architectural seams for future multi-user and collaborative state, presence, and interaction

1. a stable-enough experimental core API centered on `Machine`
2. a capable scene/layout/render/runtime pipeline
3. a real backend story via Notcurses and test doubles
4. a small but representative standard-machine library
5. a principled focus and modal interaction model
6. clear docs and examples that demonstrate the framework’s intended style
7. collaboration-aware architecture boundaries that downstream projects can safely depend on

## Release criteria

Knopper is ready for a first release when the following are true.

### Core API

- `Machine`, scene, layout, render, runtime, and backend APIs feel coherent and documented
- core traits and types compile cleanly under strict clippy settings
- the public API surface is intentionally shaped rather than accidental
- local state, shared state, and derived projection responsibilities remain clearly separated so collaboration can be layered in without breaking core assumptions

### Interaction model

- focus traversal semantics are explicit and predictable
- scope policies exist and are tested
- modal behavior is reusable and structurally expressed
- common controls do not require ad hoc focus hacks

### Standard machines

At minimum, the first release should include:

- input field
- list
- modal wrapper
- command palette
- button
- toggle or checkbox
- tabs or segmented selector
- one richer text editing surface, likely a basic textarea/editor

### Rendering and backend

- mock renderer/backend remain first-class for tests
- Notcurses backend is reliable enough for demos and examples
- cursor placement, border drawing, scrolling, overlays, and alignment work predictably

### Documentation and examples

- one overview guide for the framework model
- one guide for writing a custom machine
- one guide for composing standard machines
- one polished example app
- one focused modal/focus/navigation example
- one Notcurses-oriented visual example
- collaboration-oriented notes explaining which APIs are expected to remain compatible with future shared-state and multi-participant runtimes

### Quality gate

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
- CI remains green

## Non-goals for first release

The following are important, but should not block `0.1.0` unless they become necessary for coherence.

- full collaboration runtime and protocol integration
- complete FRP/geometric semantics beyond current scaffolding
- advanced renderer micro-optimizations everywhere
- a huge widget catalog
- generalized accessibility framework
- exhaustive terminal backend portability beyond the current abstraction strategy

## Guiding principles

1. **Preserve the machine-first model.**
   Avoid drifting toward a component API that obscures the algebraic projection model.

2. **Keep collaboration in view at every layer.**
   Even when implementing single-user behavior first, preserve the architectural separation between local state, shared state, participant-specific state, and derived scene projection.

3. **Keep semantics structural.**
   Prefer scene-expressed focus, modal, and layout behavior over control-specific hacks.

4. **Bias toward reusable standard machines.**
   New behavior should be extracted into reusable helpers when it appears in more than one place.

5. **Protect backend isolation.**
   Knopper may be tailored toward Notcurses, but the public model should stay scene/runtime-centric.

6. **Keep TDD discipline.**
   New semantics should arrive with tests at the scene, routing, runtime, and standard-machine levels.

7. **Ship a focused first release.**
   Prefer a small coherent toolkit over a broad but inconsistent one.

## Workstreams

## 1. Interaction and focus semantics

### Objective

Make focus movement, modal trapping, and scope-local traversal explicit, reusable, and runtime-driven.

### Status

Partially complete.

Implemented already:

- focus paths and focus state
- focus order derivation from scenes
- named focus scopes
- nearest-scope focus derivation
- runtime `Tab` / `Shift-Tab` traversal
- modal trapping helper primitives

### Remaining work

- add explicit focus scope policies
  - wrap
  - trap
  - passthrough
  - local-only
- define default runtime behavior at scope boundaries
- decide whether policy lives directly on `Scene::FocusScope` or in scene metadata
- formalize focus eligibility rules for hidden, disabled, or otherwise non-interactive nodes
- ensure composed standard machines derive focus order from scopes consistently
- add focus debugging/introspection helpers if needed for development

### Exit criteria

- focus traversal behavior is deterministic and documented
- nested scopes behave correctly under tests
- modal behavior is implemented primarily through shared focus policy machinery
- standard controls rely on the shared focus system rather than local special cases

## 2. Collaboration-readiness and shared-state seams

### Objective

Ensure every major subsystem remains compatible with the collaborative direction of the project, even if the full runtime lands after `0.1.0`.

### Status

Architecturally anticipated, but not yet formalized enough in implementation and release criteria.

### Remaining work

- document collaboration-relevant invariants for `Machine`
  - what belongs in local model state
  - what belongs in shared state
  - what must remain derivable from shared inputs
- audit standard machines for assumptions that implicitly hardcode single-user ownership of focus, selection, or editing state
- identify which interaction states are participant-local versus potentially shared
- define a minimal collaboration-ready contract for downstream projects
  - stable shared-state seam
  - participant-local focus seam
  - message/update patterns that will survive distributed synchronization
- add targeted docs explaining how future collaborative runtimes are expected to layer onto the current core
- where practical, prefer APIs that do not assume a single global focus forever

### Exit criteria

- the roadmap to collaboration is explicit rather than implied
- `Machine` and runtime APIs preserve the local/shared separation already present in the architecture docs
- standard machines avoid unnecessary single-user assumptions in their public contracts
- downstream IA projects can begin depending on Knopper without expecting a near-term architectural rewrite

## 3. Standard-machine ecosystem

### Objective

Expand from proof-of-concept controls into a compact but useful standard library.

### Status

Good early base.

Implemented already:

- list
- input
- list-detail composition
- command palette
- modal wrapper

### Remaining work

Priority controls for `0.1.0`:

- button
- checkbox or toggle
- tabs or segmented selector
- textarea/editor
- optional split-pane or panel composition primitive if examples need it

Cross-cutting work:

- shared control styling conventions
- common interaction messages and helper patterns
- machine composition guidelines and examples
- better child-machine dispatch ergonomics where repeated

### Exit criteria

- enough controls exist to build a meaningful demo application
- controls share a coherent interaction model
- machines compose without bespoke routing logic in every parent

## 4. Runtime and event policy

### Objective

Turn the runtime from a minimal dispatcher into a principled interaction engine.

### Status

Functional but still thin.

Implemented already:

- event routing
- message emission via activation
- focus changes
- update/effect application
- reprojection
- render and backend integration

### Remaining work

- formalize default handling for focus navigation keys beyond tabbing if needed
- define how runtime and machines cooperate on key dispatch precedence
- evaluate whether any bubbling/capture-style semantics are needed
- refine cursor and focus synchronization behavior
- ensure runtime behavior stays predictable for nested composed machines

### Exit criteria

- runtime interaction rules are documented
- default keyboard behavior is coherent across standard machines
- parent/child dispatch rules are explicit and tested

## 5. Scene algebra refinement

### Objective

Keep the scene model expressive without letting it become inconsistent or overloaded.

### Status

Strong direction, still evolving.

Implemented already:

- layout wrappers: padding, border, sized, viewport, scroll, align
- composition wrappers: stack/overlay
- semantic wrapper: focus scope
- annotations and styling metadata

### Remaining work

- review whether scene wrappers should be grouped conceptually as visual, structural, and semantic
- clarify how disabled/hidden/interactivity-related semantics are represented
- consider whether focus policy belongs in `FocusScope` directly
- add normalization expectations if scene growth starts creating ambiguous traversal or render behavior

### Exit criteria

- scene wrapper semantics are documented clearly
- interaction-related wrappers do not leak rendering concerns
- the public scene model still feels small enough to learn

## 6. Rendering and backend maturity

### Objective

Make the rendering path reliable and convincing for real demos while preserving backend abstraction.

### Status

Strong skeleton with real backend support.

Implemented already:

- render op lowering
- diffing
- backend command generation
- mock backend
- Notcurses backend with plane cache and cursor support
- border drawing and overlay support
- clipping and scrolling groundwork

### Remaining work

- validate Notcurses behavior with richer demos
- improve confidence in plane reuse/update behavior
- refine redraw heuristics as needed
- exercise alignment, scroll, border, viewport, and overlays together in integration-style tests or demos
- document backend expectations and feature-flag use

### Exit criteria

- example apps render predictably in Notcurses
- update behavior is stable across common interactions
- test doubles remain sufficient for most framework tests

## 7. Documentation and examples

### Objective

Make Knopper understandable and usable by someone other than the original implementer.

### Status

Architecture docs exist; user-facing framework docs are still sparse.

### Remaining work

Core docs to add before `0.1.0`:

- framework overview
- machine authoring guide
- scene/layout/render pipeline guide
- standard-machine composition guide
- focus and modal semantics guide
- backend/Notcurses usage notes

Examples to add before `0.1.0`:

- polished small application demo
- modal and focus traversal demo
- Notcurses-oriented visual showcase

### Exit criteria

- a new contributor can understand how to build a small app from the docs
- examples demonstrate the intended architecture rather than bypassing it

## Milestones

## Milestone 1: finish interaction semantics and collaboration-aware focus policy

### Goal

Make focus and modal behavior fully explicit and reliable.

### Deliverables

- focus scope policies implemented
- tests for nested scope behavior and policy interactions
- command palette and modal helper migrated to final shared policy model
- documented focus traversal rules
- explicit notes on how focus policy should evolve under multi-participant runtimes

### Why first

This is the main semantic gap remaining in the current architecture. It also affects every future control.

## Milestone 2: collaboration-readiness audit plus first useful standard-machine set

### Goal

Provide enough controls to build a meaningful application.

### Deliverables

- collaboration-readiness audit of core APIs and standard machines
- button
- toggle/checkbox
- tabs or segmented selector
- textarea/editor
- improved composition helpers as needed
- one demo app built from these machines

### Why second

This proves the architecture scales from primitives to an actual toolkit.

## Milestone 3: polish runtime and backend behavior

### Goal

Ensure that common app behavior feels stable and predictable.

### Deliverables

- documented runtime event policy
- refined cursor/focus behavior
- stronger Notcurses demo coverage
- integration-style tests for common interaction flows

### Why third

This turns the framework from an interesting prototype into something users can trust for experimentation.

## Milestone 4: docs and release shaping

### Goal

Package the project as a usable experimental release.

### Deliverables

- release-oriented docs
- examples
- API review for naming and cohesion
- `0.1.0` checklist
- release notes draft

### Why fourth

A first release should communicate clearly what Knopper is and how it should be used.

## Suggested execution order

1. focus scope policies
2. migrate modal and command-palette behavior onto those policies
3. perform a collaboration-readiness audit of core APIs and standard machines
4. add button and toggle/checkbox
5. add tabs or segmented selector
6. add textarea/editor
7. build a polished demo app from the standard machines
8. refine runtime event policy based on demo pressure
9. harden Notcurses behavior through demo-driven fixes
10. write first-release docs and examples, including collaboration-oriented guidance
11. review API surface and cut `0.1.0`

## Project health snapshot

### Strong areas

- machine-centered architecture
- end-to-end runtime/render/backend pipeline
- test discipline
- Notcurses integration strategy
- reusable composition patterns
- scene-derived focus foundations

### Main risks

- interaction policy complexity as more controls arrive
- scene algebra growing without enough semantic consolidation
- backend-specific pressure leaking into public APIs
- under-investment in docs/examples before release
- accidentally hardening single-user assumptions into core or standard-machine APIs

## Definition of success for `0.1.0`

Knopper `0.1.0` is successful if it can convincingly support a small terminal application with:

- multiple composed machines
- modal interactions
- focus traversal that behaves predictably
- text input and list navigation
- a real Notcurses-backed UI
- tests and docs that make the programming model legible
- a clear story for how downstream collaborative projects can depend on the framework today without expecting the core model to be overturned later

That would establish Knopper as a serious experimental foundation for the broader collaborative and FRP-oriented vision.
