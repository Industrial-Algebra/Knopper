# Scene, Layout, Render, Diff, and Backend Pipeline

## Purpose

This guide explains how a Knopper scene becomes terminal output.

It is intended to connect the user-facing machine model to the lower-level rendering pipeline so that framework users can understand:

- what a machine is projecting
- how scene structure becomes geometry
- how render ops are produced
- how diffs and backend commands are generated
- where cursor placement fits in

If the earlier guides explain how to author and compose machines, this guide explains what happens after projection.

## The big picture

At a high level, Knopper currently follows this flow:

```text
Machine state + shared state + context
-> Scene projection
-> Layout resolution
-> Render-op lowering
-> Diff
-> Backend commands
-> Backend execution
```

This is one of Knopper’s most important architectural separations.

Why it matters:

- machines stay backend-neutral
- scenes stay semantic
- layout stays explicit and testable
- render ops stay structured
- backends stay isolated
- diffs can exploit stable identity

## Step 1: scene projection

A machine projects a `Scene<Msg>` or, more generally, a `Behavior<Scene<Msg>>`.

A scene is the semantic UI description.

It expresses things like:

- text
- rows and columns
- borders
- padding
- scroll and viewport structure
- overlays and stacks
- alignment
- annotations
- focus scopes
- activation and focusability

A scene is **not** yet laid out terminal output.

It does not directly say:

- exact final coordinates of every node
- exact backend commands
- terminal-plane allocation

That is the job of later stages.

## Step 2: layout resolution

Layout turns a `Scene` into a `LayoutNode` tree with concrete geometry.

The main API is:

- `resolve_layout(scene, bounds) -> LayoutNode`

Key layout types include:

- `Rect`
- `Size`
- `LayoutNode`
- `LayoutKind`

### What layout resolves

Layout determines:

- resolved rectangles
- child positioning inside rows/columns
- padding insets
- border insets
- alignment placement
- sized constraints
- viewport structure
- scroll structure
- inherited/effective styles

### What layout preserves structurally

Some scene wrappers remain structural in layout rather than becoming visual draw commands themselves.

Examples:

- `FocusScope`
- `Align`
- `Padding`
- `Sized`
- `Viewport`
- `Scroll`
- `Border`
- `Annotated`

The important idea is:

> layout resolves geometry while preserving enough structure for render lowering to make correct decisions later.

## `Rect` and `Size`

`Rect` is the basic resolved geometry type:

```rust
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}
```

`Size` is the measurement shape used during layout:

```rust
pub struct Size {
    pub width: u16,
    pub height: u16,
}
```

Machines usually do not manipulate these directly unless they are doing something advanced, but they matter for layout understanding and cursor placement.

## `LayoutNode`

A `LayoutNode` is the resolved form of a scene node.

It contains:

- `id`
- `rect`
- `style`
- `kind`

This is important because identity is preserved through layout.

That helps with:

- focus hit resolution
- cursor computation
- diffing
- backend patch stability
- future collaboration overlays

## `LayoutKind`

`LayoutKind` mirrors the scene structure at the layout layer.

Examples include:

- `Text`
- `Row`
- `Column`
- `Stack`
- `FocusScope`
- `Align`
- `Padding`
- `Sized`
- `Viewport`
- `Scroll`
- `Border`
- `Annotated`

This is how Knopper keeps the pipeline structured rather than collapsing everything into raw drawing too early.

## Finding layout nodes

Knopper provides:

- `find_node(layout, id)`

This is particularly useful for machines that need geometry-aware behavior after layout.

The main current use is cursor placement.

## Step 3: render-op lowering

Once layout is resolved, Knopper lowers the layout tree into backend-neutral render operations.

The main API is:

- `render_ops(layout) -> Vec<RenderOp>`

Current render ops include:

- `DrawText`
- `DrawBorder`
- `Annotate`
- `SetCursor`

These are not backend-specific. They are the renderer/backend-neutral bridge between semantic layout and terminal execution.

## Why render ops exist

Render ops matter because they:

- isolate machines from backend details
- make rendering testable
- create a stable diff boundary
- let different backends translate the same semantic result

This is one of the reasons Knopper can have both:

- mock rendering/backend tests
- a real Notcurses backend

without making machines aware of backend internals.

## Structural lowering rules

Not every layout node directly becomes a visible draw op.

### Text

A `LayoutKind::Text` node lowers to `RenderOp::DrawText`.

### Border

A `LayoutKind::Border` node lowers to a `DrawBorder`, then its child continues lowering.

### Annotated

A `LayoutKind::Annotated` node lowers an `Annotate` op, then lowers its child.

### Structural wrappers

Wrappers like these are structural in render lowering:

- `FocusScope`
- `Align`
- `Padding`
- `Sized`

They affect geometry or traversal context, but do not directly emit their own visible draw ops.

## Viewport and scroll in render lowering

Viewport and scroll are particularly important because they influence clipping and translation.

### Viewport

A viewport establishes clipping.

### Scroll

A scroll wrapper adjusts the render context’s scroll offset and clipping behavior.

This means hidden content may still exist semantically and geometrically, but only a visible portion becomes render ops.

This is essential for usable scrolling behavior.

## Text clipping

Text clipping currently happens during render-op generation.

That means the layout tree may preserve more text extent than is finally drawn, especially under viewport or scroll conditions.

This is intentional.

It allows hidden content to remain revealable via scrolling.

## Step 4: diffing

Knopper diffs render ops from one frame to the next.

The main API is:

- `diff_render_ops(previous, next) -> Vec<PatchOp>`

Current patch forms are:

- `PatchOp::Insert(RenderOp)`
- `PatchOp::Update(RenderOp)`
- `PatchOp::Remove(NodeId)`

### Why diffing happens here

Diffing render ops instead of terminal bytes or semantic machine state is a practical middle ground.

It gives Knopper:

- identity-aware updates
- backend-neutral patching
- reduced redraw work
- simpler testing

### Identity matters

Diffing relies heavily on stable `NodeId` identity.

If your machine changes IDs unnecessarily, the diff sees churn and cannot preserve update stability well.

That is one reason stable IDs are so important in machine authoring.

## Step 5: backend command generation

Patches are lowered into backend commands.

The main API is:

- `backend_commands(previous, patches) -> Vec<BackendCommand>`

Current backend command forms include:

- `DrawText`
- `DrawBorder`
- `Annotate`
- `SetCursor`
- `ClearNode`
- `ClearRect`

### Update behavior

Current update behavior is roughly:

- `Insert` -> draw
- `Update` -> clear old rect, then redraw
- `Remove` -> clear old rect, then clear node identity

This keeps the backend layer simple and explicit.

## Step 6: backend execution

Finally, a backend executes the backend commands.

The backend abstraction is:

- `TerminalBackend`

Current backend implementations include:

- `MockBackend`
- feature-gated `NotcursesBackend`

## `MockBackend`

Use `MockBackend` for tests and pipeline validation.

It records executed commands and tracks backend state in a structured way.

This is one of the reasons the Knopper pipeline is easy to test without depending on a live terminal backend.

## `NotcursesBackend`

The Notcurses backend is feature-gated and uses real Notcurses integration.

It is responsible for translating backend commands into concrete terminal behavior, including:

- plane usage and reuse
- text drawing
- border drawing
- cursor updates
- clearing and refresh behavior

Importantly, Notcurses remains an implementation detail of the backend layer, not the public machine/scene model.

## Cursor placement in the pipeline

Cursor placement is not hardcoded in the renderer.

Instead, the current model is:

1. machine projects a scene
2. layout resolves node geometry
3. machine may compute cursor position from the resolved layout
4. runtime passes that cursor position to the backend

The relevant machine hook is:

- `cursor_position(model, shared, ctx, layout)`

This is how controls like `InputMachine` and `TextareaMachine` place the cursor correctly.

### Why this is good

This keeps cursor behavior:

- semantic
- layout-aware
- testable
- backend-neutral

It also aligns well with future collaboration overlays, since stable semantic anchors remain available.

## Runtime integration

The runtime ties the whole pipeline together.

Important runtime methods include:

- `layout(bounds)`
- `render_ops(bounds)`
- `diff(bounds)`
- `render(renderer, bounds)`
- `render_to_backend(backend, bounds)`
- `render_to_backend_with_cursor(...)`
- `render_to_backend_auto_cursor(...)`

This is the main path by which machine projection becomes terminal-visible output.

## A practical walkthrough

Suppose a machine projects:

```rust
Scene::border(
    root_id,
    Scene::sized(
        inner_id,
        SizeConstraint::width(12),
        Scene::text(label_id, "Run").focusable(),
    ),
)
```

Then the pipeline roughly does this.

### Scene

Semantic structure:

- border
- sized region
- text node

### Layout

Resolves:

- exact border rect
- exact inner rect
- effective style inheritance
- child geometry inside the border

### Render lowering

Produces something like:

- `DrawBorder { id: root_id, ... }`
- `DrawText { id: label_id, content: "Run", ... }`

### Diff

Compares these ops to previous-frame ops.

### Backend commands

Lowers the diff into draw/clear commands.

### Backend

Executes the commands on the target backend.

That is the full semantic-to-terminal path.

## What machine authors should care about most

As a machine author, you do **not** usually need to work with every pipeline stage directly.

The most important practical implications are:

### 1. Project semantic scenes, not backend details

Stay at the scene layer.

### 2. Use stable IDs

Stable identity improves focus, diffing, and backend updates.

### 3. Use structural scene wrappers intentionally

Wrappers like border, padding, sized, viewport, scroll, and focus scope all influence later pipeline stages.

### 4. Use `cursor_position(...)` for text-entry controls

Do not bury cursor logic in rendering assumptions.

### 5. Test through intermediate layers when useful

You can test:

- scene shape
- layout nodes
- render ops
- patches
- backend commands

This is one of Knopper’s strengths.

## Common mistakes to avoid

### 1. Thinking a scene is already laid out terminal output

A scene is semantic input to later stages, not final drawing.

### 2. Using unstable IDs

This makes diffing and focus worse and will also hurt future collaboration overlays.

### 3. Treating backend behavior as machine behavior

Backends execute rendering; machines project semantics.

### 4. Ignoring clipping and scroll structure

Viewport and scroll matter at render lowering time, not just in visual intuition.

### 5. Assuming cursor placement belongs in the backend

Cursor placement belongs at the machine/layout boundary.

## Recommended code references

Useful files to study with this guide:

- `src/layout.rs`
- `src/render.rs`
- `src/diff.rs`
- `src/backend.rs`
- `src/runtime.rs`
- `docs/architecture/04-runtime-pipeline.md`
- `docs/architecture/05-rendering-model.md`

## Bottom line

If you remember one sentence from this guide, remember this:

> in Knopper, machines project semantic scenes, layout resolves geometry, render lowering produces backend-neutral ops, diff computes change, and the backend executes the result.
