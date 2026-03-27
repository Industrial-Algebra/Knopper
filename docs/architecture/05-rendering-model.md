# Knopper Architecture: Rendering Model

## Status

Draft.

## Purpose

This document defines the first-pass rendering architecture for Knopper.

Knopper is a terminal-native framework, but rendering must remain separate from scene semantics. Notcurses is the initial backend, not the public API.

---

## Core principle

Knopper rendering should follow this separation:

```text
Scene -> Layouted Scene -> Render Ops -> Backend Lowering -> Commit
```

This separation is required for:
- testability,
- backend isolation,
- efficient diffing,
- future backend flexibility,
- lawful composition of scene and layout passes.

---

## Rendering layers

## 1. Declarative scene

The output of machine projection.

Properties:
- terminal-native,
- identity-rich,
- interactive,
- backend-neutral,
- not yet layout-resolved.

This is the author/runtime semantic representation.

---

## 2. Layouted scene

The result of layout resolution and style inheritance.

Properties:
- absolute or resolved bounds,
- clipping regions,
- effective style values,
- z-order/layer placement,
- hit regions for interaction,
- collaboration overlay positions.

This is the structured rendering input to diffing.

---

## 3. Render operations

Backend-neutral draw/update instructions.

Examples:
- draw text run,
- clear region,
- fill region,
- draw border,
- create/update/destroy surface,
- place cursor,
- update style region,
- composite overlay layer.

Render ops are the bridge between semantic layout and backend execution.

---

## 4. Backend lowering

Render ops are translated into concrete backend calls.

For Notcurses this may include:
- plane creation and reuse,
- plane movement and resize,
- grapheme/cell writes,
- style and color updates,
- z-order configuration,
- visual effects supported by Notcurses.

---

## 5. Commit

The backend performs the terminal render commit.

This likely remains serialized and tightly controlled.

---

## Why Notcurses must stay isolated

Knopper should not let public Machine or Scene APIs depend on:
- planes,
- low-level cells,
- backend handles,
- Notcurses-specific lifetimes or mutation semantics.

Reasons:
- keeps Machines testable,
- prevents backend leakage into app logic,
- preserves possible future backend options,
- avoids coupling scene semantics to terminal implementation quirks.

---

## Renderer responsibilities

A renderer backend should be responsible for:
- resource allocation and reuse,
- terminal capability adaptation,
- grapheme and style emission,
- clipping and z-order realization,
- cursor placement,
- final commit.

A renderer backend should not own:
- machine logic,
- scene semantics,
- message routing,
- state updates.

---

## Diffing strategy

Knopper should prefer structured diffing over full redraw.

Good diff inputs:
- previous layouted scene,
- current layouted scene,
- stable node identity,
- effective styles,
- content runs,
- layout geometry.

Diffing goals:
- minimize backend work,
- preserve long-lived surfaces where valuable,
- avoid unnecessary redraw of unchanged regions,
- exploit stable identity and deterministic structure.

---

## Surface/plane management

Although Notcurses may use planes internally, Knopper should model backend-neutral surfaces or layers at the render-op/backend boundary.

A useful conceptual model is:
- scene nodes imply potential surfaces,
- layouted scene determines placement,
- diff decides reuse/create/destroy,
- backend maps surfaces to planes.

This makes plane reuse a renderer optimization, not a semantic concern.

---

## Text rendering concerns

Terminal rendering has text-specific challenges that Knopper must account for early.

Important concerns include:
- grapheme segmentation,
- unicode width,
- wide glyphs,
- combining marks,
- clipping of styled text runs,
- wrapping and truncation,
- terminal-safe cursor placement.

These concerns belong primarily in layout and render-op generation, not in machine authorship.

---

## Style application

Style resolution should occur before backend lowering.

This means the renderer should receive effective styles rather than needing to reconstruct:
- inheritance,
- theme overlays,
- focus styling,
- collaboration styling.

This keeps renderer backends simpler and more deterministic.

---

## Collaboration overlays in rendering

The rendering model must support overlays such as:
- remote cursors,
- remote selections,
- participant markers,
- sync status surfaces.

These should likely be represented as explicit overlay/layer constructs in the layouted scene and render-op model.

This avoids coupling collaboration directly to text drawing internals.

---

## Cursor model

Knopper should distinguish:
- semantic cursor/selection in scenes,
- rendered terminal cursor placement in the backend.

The runtime/layout layer should determine which semantic cursor becomes the active terminal cursor for the local participant.

Remote cursors should be rendered as overlays, not terminal cursor ownership.

---

## Performance guidance

Knopper should optimize rendering around:
- stable identity,
- subtree diffing,
- text run reuse,
- bounded allocations,
- batched render ops,
- selective backend mutation.

Parallel preparation via `rayon` is encouraged where valid, but the final backend commit should be treated conservatively.

---

## Debugging and inspection

The rendering model should support inspection of:
- layouted scene snapshots,
- diff summaries,
- render-op streams,
- backend surface/plane mappings.

This is important both for framework development and application troubleshooting.

---

## First implementation recommendation

Initial rendering implementation should support:
- text rendering,
- row/column/stack layout lowering,
- sized and padded regions,
- borders,
- simple overlays,
- cursor placement,
- diff-based redraw.

Avoid over-optimizing too early; preserve architecture first.

---

## Open follow-up questions

1. exact backend-neutral render-op algebra
2. explicit surface abstraction vs direct region ops
3. text wrapping/truncation semantics in layout vs renderer
4. how much Notcurses advanced media support belongs in v0
5. renderer diagnostics API shape

These should be addressed in later ADRs.
