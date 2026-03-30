# Walkthrough: the Knopper Demo Workspace

## Purpose

This guide walks through the current Knopper demo workspace and explains how it is built from machines, scenes, layout structure, and derived state.

It is intended to connect the earlier guides to a concrete example.

If you have already read:

- [01-writing-a-machine.md](01-writing-a-machine.md)
- [02-composing-machines.md](02-composing-machines.md)
- [03-focus-and-modal-semantics.md](03-focus-and-modal-semantics.md)
- [04-scene-layout-render-pipeline.md](04-scene-layout-render-pipeline.md)

then this guide shows how those ideas come together in a small application-like workspace.

## Where the demo lives

Main implementation files:

- `src/demo.rs`
- `src/main.rs`

Run it with:

```bash
cargo run
```

The current binary prints a rendered snapshot of the demo scene rather than running a full interactive terminal loop.

That is still useful because it exercises:

- machine composition
- scene projection
- layout and render lowering
- derived status state
- cursor placement

## What the demo contains

The demo workspace currently composes these standard machines:

- `TabsMachine`
- `ToggleMachine`
- `ButtonMachine`
- `TextareaMachine`
- `ListMachine`

And adds parent-level derived state:

- a status line summarizing the workspace state

Conceptually, the workspace looks like this:

```text
Header
Tabs
Toggle + Button controls
Textarea + Task list body
Status line
```

## The demo machine

The demo is itself just another machine:

- `DemoMachine`

This is an important Knopper idea:

> applications are machines too.

A larger workspace is not a different abstraction from a small control. It is just a machine composed from other machines.

## `DemoState`

The demo state stores child model state plus parent-derived state:

- `tabs: TabsState`
- `toggle: ToggleState`
- `button: ButtonState`
- `textarea: TextareaState`
- `list: ListState`
- `status: String`

This is a strong example of composed state design.

### Child-owned state

These fields belong to children because they are intrinsic to those controls:

- tab selection
- toggle checked state
- button activation count
- textarea cursor/content state
- list selection/scroll state

### Parent-owned derived state

`status` belongs in the parent because it summarizes multiple child states.

That is exactly the kind of state that should live in a composed parent machine.

## `DemoMsg`

The demo message type lifts child messages into parent space:

- `Tabs(TabsMsg)`
- `Toggle(ToggleMsg)`
- `Button(ButtonMsg)`
- `Textarea(TextareaMsg)`
- `List(ListMsg<()>)`

This is the standard Knopper composition pattern.

Each child remains reusable because the parent wraps child messages rather than changing child APIs.

## `DemoContext`

`DemoContext` contains configuration for the whole workspace and its children.

It includes:

- overall workspace width and height
- child contexts for tabs, toggle, button, textarea, and list
- list items for the task list

This is a good demonstration of how application-level configuration can be distributed to child machines through child-specific contexts.

## Child machine construction

`DemoMachine` stores child machine instances as fields:

- `tabs: TabsMachine`
- `toggle: ToggleMachine`
- `button: ButtonMachine`
- `textarea: TextareaMachine`
- `list: ListMachine<...>`

That lets the parent:

- update children
- project children
- keep child-specific helper methods available if needed

The list machine is configured with a custom row renderer so the workspace can shape how task rows appear.

## Derived parent status

The demo has a helper method:

- `sync_status(...)`

This method derives a status string from:

- selected tab
- toggle state
- button activation count
- committed textarea value
- selected list item index

This is a good example of an app-level aggregation step.

It shows how parent machines can give semantic meaning to multiple child states together.

In other words:

- child machines manage local control behavior
- the parent derives workspace meaning

## Update flow

The update logic in `DemoMachine` follows a classic composition shape.

For each child message:

- call `update_child(...)`
- update the corresponding child model field
- lift child effects into `DemoMsg`
- then recompute parent status

Example pattern:

```rust
DemoMsg::Toggle(msg) => update_child(
    &self.toggle,
    &mut model.toggle,
    msg,
    &ctx.toggle,
    &DemoMsg::Toggle,
)
```

This is one of the cleanest examples in the codebase of how `update_child(...)` should be used in practice.

## Projection flow

Projection uses `project_child(...)` for each child machine.

Example shape:

```rust
let tabs = project_child(&self.tabs, &model.tabs, &(), &ctx.tabs, &DemoMsg::Tabs);
let toggle = project_child(&self.toggle, &model.toggle, &(), &ctx.toggle, &DemoMsg::Toggle);
```

Each child scene is projected into the parent message space, then assembled into a larger scene.

This is one of the core Knopper composition patterns.

## Scene structure of the workspace

At a high level, the demo scene is built from:

- a header text node
- the tabs row
- a controls row
- a body row
- a status line

### Controls row

The controls row contains:

- toggle
- button

### Body row

The body row contains:

- textarea on the left
- bordered task list on the right

### Root wrapping

The whole workspace is wrapped in:

- a focus scope with `Passthrough`
- a border
- a sized root
- a top-level column

This gives the demo a coherent application-like surface and explicit focus semantics.

## Why `Passthrough` focus policy?

The demo root uses:

- `FocusScopePolicy::Passthrough`

This is a sensible app-level policy because the workspace is one meaningful region, but its local traversal should not become an isolated modal trap.

That means:

- local structure matters
- but focus can still progress through the broader scene-derived order

This is a useful contrast with modal-like controls such as the command palette, which use stricter local semantics.

## Stable IDs in the demo

The demo assigns explicit IDs across child contexts and parent structure.

This matters for:

- focus
- activation
- diffing
- cursor placement
- future presence overlays

Even though the demo is not yet a collaborative app, it already follows the identity discipline needed for future collaboration overlays.

## The textarea cursor

The demo machine overrides `cursor_position(...)` and delegates to the textarea child.

That is a very instructive pattern.

It means:

- the parent app still owns the final cursor hook
- but it can delegate to the child that actually understands the text cursor

This is the right kind of separation for composed text-centric workspaces.

## The task list

The demo uses `ListMachine<String, (), ...>` with a custom row renderer.

That shows an important Knopper pattern:

> standard machines are reusable, but parents can still customize their semantic row projection.

The task list is not hardcoded into the list machine itself. The parent supplies how items should render.

## Why the demo matters architecturally

The demo is more than a screenshot generator.

It validates several project goals at once.

### 1. Machines compose cleanly

The workspace is built entirely from the same machine abstraction used for controls.

### 2. Parent semantics layer cleanly over child semantics

The derived status line shows how the parent can summarize multiple controls without mutating child abstractions.

### 3. Scene structure is expressive enough for app layout

The workspace uses rows, columns, borders, sizing, and focus scope in a natural way.

### 4. Cursor placement stays machine-driven

The textarea cursor remains semantic and layout-aware.

### 5. The result flows through the full pipeline

The demo reaches:

- scene projection
- layout
- render lowering
- diffing
- render-op inspection in the binary

## What `src/main.rs` does

The current binary is intentionally simple.

It:

1. constructs `Runtime<DemoMachine>`
2. sends a few demo messages
3. renders to `RenderOp`s
4. filters text ops
5. prints a snapshot
6. prints the derived cursor position

This is useful because it demonstrates the framework without requiring a full interactive loop yet.

It also produces a stable development artifact for quickly checking composition output.

## Example walkthrough of the scripted interaction

The current `main.rs` script does things like:

- toggle shared mode on
- press the sync button once
- select the `Notes` tab
- type `Knopper`
- insert a newline
- type `demo`
- commit the textarea

That is why the snapshot shows:

- `shared:true`
- `syncs:1`
- `tab:Notes`
- `note:Knopper | demo`

This is a compact but useful demonstration of derived workspace state.

## Collaboration framing

Even though the demo is currently local-first, it already reflects the collaboration-aware architecture.

### Local-first state in the demo

Examples:

- textarea cursor behavior
- local list selection
- local toggle/button interaction

### State that could become shared later

Examples:

- textarea content/commit semantics
- selected task semantics
- active tab or workspace mode, depending on app design
- sync-status meaning in a collaborative app

### Why this matters

The demo is not pretending to be a final collaborative workspace.

Instead, it demonstrates that the current machine/composition model can support the kinds of application structure that future collaborative IA projects will need.

## How to study the demo productively

A good reading order is:

1. `DemoContext`
2. `DemoState`
3. `DemoMsg`
4. `sync_status(...)`
5. `update(...)`
6. `project_once(...)`
7. `cursor_position(...)`
8. `src/main.rs`

That order lets you move from app surface to behavior to rendering snapshot.

## Good extension exercises

If you want to learn Knopper by modifying the demo, good next steps are:

### 1. Add another standard machine

For example:

- another toggle
- a second button
- a command palette overlay

### 2. Make the status line richer

For example:

- include committed task details
- include focus information
- include synthetic participant/presence markers later

### 3. Add a modal

Wrap part of the workspace with the reusable modal helper and test focus behavior.

### 4. Move some semantics into `Shared`

As a design exercise, imagine which demo state would become shared in a collaborative variant.

This is a good way to internalize Knopper’s collaboration-aware state separation.

## Common takeaways

When reading this demo, the most important lessons are:

- apps are machines too
- composition is ordinary machine authoring
- parent machines can derive higher-level semantics from child state
- focus and layout remain structural
- cursor placement can be delegated cleanly
- the rendering pipeline remains backend-neutral up to the backend boundary

## Recommended code references

Study these together:

- `src/demo.rs`
- `src/main.rs`
- `src/standard/tabs.rs`
- `src/standard/toggle.rs`
- `src/standard/button.rs`
- `src/standard/textarea.rs`
- `src/standard/list.rs`

## Bottom line

If you remember one sentence from this guide, remember this:

> the demo workspace shows that Knopper’s machine model is already strong enough to assemble a small application from reusable controls, derived parent semantics, structural focus rules, and a backend-neutral rendering pipeline.
