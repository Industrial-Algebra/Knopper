# Knopper Architecture: Runtime Pipeline

## Status

Draft.

## Purpose

This document defines the first-pass runtime pipeline for Knopper.

The runtime is responsible for turning raw input, async events, and collaboration updates into local/shared state changes and finally into rendered terminal output.

Knopper's runtime should be designed as an explicit transformation pipeline, strongly informed by Orlando's transducer-oriented composition model.

---

## Core pipeline

Conceptually:

```text
Raw Input / Async Events / Sync Events
-> Normalize
-> Enrich with Context
-> Route by Focus / Identity / Address
-> Lift into Machine Messages
-> Update Local and Shared State
-> Recompute Derived Behaviors
-> Project Scene
-> Annotate / Overlay / Theme
-> Resolve Layout
-> Diff
-> Generate Render Ops
-> Lower to Backend
-> Commit
```

This is the semantic pipeline Knopper should make explicit.

---

## Pipeline stages

## 1. Event ingestion

Inputs may come from:
- keyboard,
- mouse,
- paste,
- terminal resize,
- timers/ticks,
- async task completion,
- collaboration sync updates,
- system/runtime notifications.

These sources should be normalized into a common runtime event stream.

---

## 2. Normalization

Raw backend-specific events should be converted into backend-neutral input forms.

Examples:
- Notcurses key codes -> semantic key events,
- terminal resize -> size event,
- async task output -> completion event,
- remote delta -> sync event.

This stage should be isolated from machine logic.

---

## 3. Context enrichment

Normalized events may need additional runtime context before routing.

Examples:
- current focus target,
- active machine path,
- terminal capability info,
- participant/session metadata,
- current viewport or layout references,
- shared-state sync status.

This should be additive and not mutate the underlying semantic meaning unnecessarily.

---

## 4. Routing

Events must be routed to their semantic destination.

Routing factors include:
- focused node,
- focus scope,
- node identity,
- machine boundary,
- scene annotations,
- collaboration target metadata.

Routing should be deterministic and testable.

---

## 5. Message lifting

After routing, runtime events become machine-level typed messages.

This stage may use:
- node-bound message descriptors,
- machine routing tables,
- prisms for child machine message embedding,
- runtime adapters for system messages.

The goal is to present each Machine with typed semantic intents.

---

## 6. Update

Routed machine messages are applied to local state and may also generate shared-state mutations or effects.

Expected outputs from update:
- mutated local model,
- effect values,
- optional shared-state operations,
- focus requests,
- spawned async work,
- subscriptions.

This stage is where hybrid message-driven state evolution occurs.

---

## 7. Reactive recomputation

After updates, derived behaviors must be allowed to propagate.

This includes:
- local derived state,
- shared-state-derived projections,
- focus and selection derivations,
- animation/timer-derived values,
- collaboration overlays.

This stage is where Cliffy is central.

---

## 8. Scene projection

Machines project `Behavior<Scene<Msg>>`.

The runtime should then sample or observe the relevant scene values needed for rendering and routing.

Projection should remain renderer-independent.

---

## 9. Scene annotation and overlay passes

Before layout, Knopper may enrich the scene with runtime-level overlays or metadata.

Examples:
- theme application,
- focus decorations,
- participant overlays,
- sync status indicators,
- remote cursor layers,
- debug instrumentation.

This stage is a good fit for explicit transformation passes.

---

## 10. Layout resolution

The declarative scene is transformed into a layouted scene with resolved:
- bounds,
- clipping,
- effective styles,
- z-order,
- focus hit regions,
- collaboration overlay placement.

Layout should be independent from the renderer backend as much as practical.

---

## 11. Diffing

The runtime should compare the previous and current layouted scenes.

Diffing should aim to:
- minimize backend work,
- preserve stable surfaces/planes where possible,
- exploit node identity,
- avoid full redraws when not required.

Diffing should operate on structured scene/layout representations, not raw terminal buffers.

---

## 12. Render op generation

From the diffed layouted scene, Knopper generates backend-neutral render operations.

Examples:
- draw text run,
- clear region,
- apply style region,
- move cursor,
- create/update/destroy surface,
- draw border,
- update overlay layer.

This stage isolates renderer backends from higher-level scene semantics.

---

## 13. Backend lowering and commit

Finally, render ops are lowered to backend-specific operations.

For the initial backend:
- Notcurses plane operations,
- style/cell updates,
- cursor placement,
- commit/render call.

This final stage should be carefully isolated and likely serialized.

---

## Runtime responsibilities beyond rendering

The runtime also owns:
- machine instance management,
- subscription registration,
- async task lifecycle,
- focus state management,
- tick scheduling,
- shared-state integration,
- resource cleanup.

It is therefore not just a render loop; it is the execution engine for Machines.

---

## Orlando influence

Knopper should adopt Orlando's transducer mindset even if it does not literally reuse every Orlando type directly.

The runtime pipeline should emphasize:
- composable passes,
- bounded intermediates,
- explicit transformations,
- law-friendly composition,
- potential pass fusion.

This should make the pipeline readable, testable, and optimizable.

---

## Parallelism guidance

Knopper should use `rayon` where CPU-bound stages benefit from parallelism.

Promising candidates:
- subtree layout computation,
- text measurement/cache work,
- diff preparation,
- overlay annotation over large scenes,
- render op batching.

Likely serialized stages:
- final Notcurses mutation,
- terminal I/O commit,
- some focus/runtime coordination paths.

Parallelism should be applied carefully and measured.

---

## Debuggability requirements

Because the runtime is a pipeline, Knopper should make intermediate stages inspectable.

Useful diagnostics:
- normalized event traces,
- routing decisions,
- projected scene snapshots,
- layouted scene dumps,
- diff summaries,
- render op traces,
- sync event traces.

This will be important for both framework development and user applications.

---

## Testing implications

Each pipeline stage should ideally be testable in isolation.

Examples:
- normalization tests,
- routing tests,
- update/effect tests,
- projection sampling tests,
- layout tests,
- diff tests,
- render op generation tests.

This argues strongly for keeping stages as explicit typed transformations.

---

## First implementation recommendation

Start with a narrow but honest pipeline:
- input normalization,
- focus-aware routing,
- machine update,
- scene projection,
- basic annotation pass,
- layout,
- diff,
- render ops,
- Notcurses commit.

Add async and collaborative sync sources incrementally without compromising the model.

---

## Open follow-up questions

1. exact runtime ownership model for machine state and behaviors
2. exact effect execution model
3. push-based vs sampled rendering loop semantics
4. scheduling of animation/tick-driven projections
5. transport abstraction for shared-state sync events

These should be resolved in later ADRs.
