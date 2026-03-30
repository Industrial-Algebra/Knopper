# Knopper textarea/editor first-pass design

## Purpose

This document defines the intended shape of a first-pass textarea/editor control for Knopper.

The aim is to add a useful multiline text-editing surface to the standard-machine ecosystem without accidentally presenting a purely local text field as Knopper’s final collaborative editor abstraction.

## Scope

This design is for a `0.1.0`-appropriate editor machine.

It should be:

- useful for local multiline editing
- composable with the existing machine/runtime/focus system
- explicit about what is local-first
- careful not to collapse future collaborative editing concerns

It is **not** yet intended to be:

- a CRDT-backed collaborative editor
- a full rope/gap-buffer editing engine
- a final rich-text or structural-code editor design

## Design goals

The first-pass textarea/editor should provide:

- multiline local text editing
- local cursor movement across rows and columns
- insertion and backspace behavior
- enter/newline insertion
- bordered and sized rendering
- viewport-aware rendering support through existing scene wrappers
- machine-derived cursor placement
- stable semantic node IDs suitable for future presence overlays

## Collaboration constraints

This control must be designed in line with the collaboration-readiness guide.

### What may remain local-first in `0.1.0`

- cursor position
- preferred column tracking
- local in-progress editing mechanics
- local scroll offset if the editor owns viewport behavior
- committed snapshot convenience state if needed for demos

### What should not be implied by the API

The API should **not** imply that:

- all text content is forever local-only
- there is one globally authoritative cursor
- remote presence should replace local focus
- renderer/backend state is part of collaboration

### Future collaborative editor split

A richer collaborative editor will likely separate:

- shared document content
- participant-local cursor/composition state
- participant-published cursor/selection state
- derived presence overlays

The first-pass textarea should leave conceptual room for that split.

## Proposed machine shape

## `TextareaState`

Local-first model state.

Likely fields:

- `value: String`
- `cursor_row: usize`
- `cursor_col: usize`
- `preferred_col: Option<usize>`
- `committed: Option<String>`

Rationale:

- `value` keeps the first implementation simple and useful
- `cursor_row` and `cursor_col` make multiline behavior explicit
- `preferred_col` supports intuitive up/down movement later if added
- `committed` is optional convenience state for local workflows/tests

## `TextareaMsg`

First-pass message surface should likely include:

- `Insert(char)`
- `Backspace`
- `Newline`
- `MoveLeft`
- `MoveRight`
- `MoveUp`
- `MoveDown`
- `Commit`

This is enough to make the control useful without overcommitting to a complex editing algebra too early.

## `TextareaContext`

Likely fields:

- `root_id: NodeId`
- `text_id_base: NodeId` or similar line identity base
- `width: u16`
- `height: u16`
- `placeholder: String`

Optional future-facing fields could include:

- viewport ownership policy
- visual line wrapping mode
- styling options

For the first pass, fixed size plus multiline content rendering is sufficient.

## Rendering model

The first-pass textarea should project through the existing scene algebra.

Recommended approach:

- render a bordered root
- render a sized viewport region
- render content as a column of text lines
- keep line IDs stable via a line base ID plus line index
- let existing layout/render pipeline handle the multiline scene

### Placeholder behavior

If the value is empty:

- render placeholder text as a single line or simple placeholder body
- keep cursor at origin of the editable field

### Scrolling

For the first pass, there are two acceptable options:

1. minimal editor without internal scrolling, relying on fixed-height clipping
2. simple local scroll offset in model once necessary

Recommendation:

Start with **no internal scroll ownership** unless implementation pressure makes it necessary immediately.

That keeps the first version simpler and avoids mixing too many concerns at once.

## Cursor semantics

Cursor placement should remain machine-derived from final layout, consistent with the current Knopper model.

That means:

- editor projects stable line nodes
- runtime resolves layout
- editor computes cursor from the selected line node geometry plus local column

This is a strong fit for future remote cursor overlays too, because it keeps semantic anchors explicit.

## Key handling

The first-pass textarea should map:

- character keys to insertion
- `Enter` to newline insertion
- `Backspace` to backward deletion
- arrow keys to cursor movement
- optionally `Ctrl+Enter` to commit, if plain `Enter` is reserved for newline

Recommendation:

Use:

- `Enter` => `Newline`
- `Ctrl+Enter` => `Commit`

That better matches multiline editing expectations.

## Stable identity expectations

The editor should preserve stable IDs for:

- root surface
- content container
- each visible or semantic line
- optional cursor anchor line

This matters for future:

- remote cursor attachment
- remote selection highlighting
- conflict markers
- attention indicators on editor regions

## Presence readiness

Even in a local-first first pass, the editor should be designed so presence overlays could later attach to:

- line nodes
- text surface root
- semantic cursor anchors

It is acceptable that remote presence is not yet implemented.

It is not acceptable to make such overlays conceptually impossible.

## Tradeoffs accepted for `0.1.0`

The first-pass textarea may:

- store all text in one `String`
- use simple line splitting
- use local-only cursor state
- omit selection ranges initially
- omit undo/redo
- omit syntax awareness
- omit remote presence rendering

These are acceptable simplifications for the first release.

## Non-goals for first pass

- collaborative merging semantics inside the control
- remote cursor rendering built in
- structural text model or rope optimization
- generalized editing transactions
- visual wrapping engine beyond simple line splitting

## Success criteria

The first-pass textarea/editor is successful if it:

- supports useful local multiline editing
- integrates cleanly with Knopper focus/runtime/layout/render systems
- exposes machine-derived cursor placement
- keeps IDs stable enough for future overlays
- does not misrepresent itself as the final collaborative editor abstraction

## Recommended implementation order

1. add `TextareaState`, `TextareaMsg`, `TextareaContext`, `TextareaMachine`
2. implement multiline insertion/backspace and left/right movement
3. implement line-aware cursor row/column computation
4. add up/down movement
5. render bordered multiline content with stable line IDs
6. implement cursor placement via resolved layout
7. add tests covering editing, movement, rendering, and cursor placement

## Documentation note for release

When this machine is introduced, the docs should explicitly describe it as:

> a local-first multiline text editing control suitable for `0.1.0`, with architecture intentionally shaped to leave room for richer collaborative editors later.
