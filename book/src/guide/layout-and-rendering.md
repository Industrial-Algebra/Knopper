# Layout and Rendering

The pipeline from scene to screen, and what it costs.

## The pipeline

```
Scene<Msg>
  → resolve_layout(scene, bounds)   LayoutNode tree (rects per node)
  → render_ops(layout)              Vec<RenderOp>  (DrawText / DrawBorder / Annotate / SetCursor)
  → diff_render_ops(prev, next)     Vec<PatchOp>   (Insert / Update / Remove, id-keyed)
  → backend_commands(prev, patches) Vec<BackendCommand>
  → backend.execute(&commands)      commit
```

Every stage is pure and separately testable. `Runtime` exposes each rung:
`layout(bounds)`, `render_ops(bounds)`, `diff(bounds)`, and the committing
`render*` paths.

## Layout

`resolve_layout` walks the scene under a `Rect` budget. Columns/rows split
space, `sized` constrains, `viewport` clips, `scroll` offsets, `align` anchors.
Layout is a pure function of `(scene, bounds)` — resize events re-run it from
the same scene.

## Diffing

Patches are keyed by `NodeId`. Unchanged ids produce no patch; new ids insert;
changed content updates; vanished ids remove. **Stable ids are what make
frames cheap** — derive them from data identity (row index, item key), never
from render order.

`Runtime::diff(bounds)` is **read-only**: it compares against the last
committed frame but doesn't advance it. The committing paths
(`render`, `render_to_backend*`) advance the baseline. A host that reads
`diff()` twice without committing sees the same cumulative patch set twice.

## Cost profile (measured)

Per-commit work is O(scene size) through layout/render, and the diff is
currently O(n²) in op count (quadratic at thousands of ops — measured in the
[Tier 0 validation](../design/streaming-append.md)). Sub-millisecond at
hundreds of nodes; an append-optimized diff is scheduled for 0.1.x. Project
the visible window (viewport-bounded scenes) for large datasets.

## Backends

- `MockBackend` — in-memory, records commands; the test backend
- `NotcursesBackend` — real terminal rendering behind `--features notcurses`
  (system notcurses ≥ 3.0.11)

See [Backends](../api/backends.md) for the trait.
