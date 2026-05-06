# Reusable Composition Patterns from `demo_ui`

## Purpose

This guide explains the small scene-composition helper layer extracted from the demo into:

- `src/demo_ui.rs`

These helpers are not full standard machines.

Instead, they demonstrate an important middle layer in Knopper:

- below reusable machines
- above raw `Scene` node assembly

That middle layer is often the right place for app-specific or domain-specific UI structure.

## Why this layer exists

When building a real Knopper application, you will often have UI structure that is:

- reused in multiple places
- more semantic than raw `Scene::row(...)` / `Scene::column(...)`
- not yet stable enough to deserve a public standard-machine API

Examples from the demo:

- a framed titled surface
- a participant-presence strip
- a labeled detail row
- wrapped/truncated text formatting for detail panes

Extracting these patterns keeps parent-machine projection code readable without prematurely turning everything into a machine.

## What lives in `src/demo_ui.rs`

Current helpers include:

- `PresenceTone`
- `PresenceCue`
- `PanelFill`
- `PanelTheme`
- `focus_style()`
- `shell_theme()`
- `panel_theme(...)`
- `reading_theme()`
- `status_theme()`
- `presence_style()`
- `presence_strip(...)`
- `panel_header(...)`
- `panel_background(...)`
- `panel_chrome(...)`
- `panel_chrome_with_fill(...)`
- `surface_panel(...)`
- `surface_panel_with_theme(...)`
- `surface_panel_with_fill(...)`
- `bounded_surface_panel(...)`
- `bounded_surface_panel_with_theme(...)`
- `bounded_surface_panel_with_fill(...)`
- `split_columns(...)`
- `master_detail(...)`
- `app_shell(...)`
- `app_shell_with_theme(...)`
- `app_shell_with_fill(...)`
- `labeled_value(...)`
- `wrap_text_lines(...)`
- `truncated_wrapped_lines(...)`

## Pattern 1: framed app surfaces

The newer `bounded_surface_panel(...)` helper is the preferred first-pass primitive when a panel needs an explicit height contract and a clipped body region.

Under the hood, panels now separate into:

- `panel_header(...)`
- `panel_chrome(...)`
- `PanelTheme`

The current emphasis is a solid panel layout and chrome system built from explicit styles for:

- border/frame
- body background
- header background
- title
- subtitle / muted metadata

This gives us a cleaner way to iterate on panel hierarchy first, while preserving `PanelFill` as an optional future path for richer Notcurses-native fills and rendering surfaces.

Use `surface_panel(...)` when you want a consistent application surface with:

- border
- padding
- section title
- subtitle
- presence strip
- body scene

Shape:

```rust
let notes = surface_panel(
    1_000_u64,
    40,
    "Notes",
    "Editing shared draft",
    &notes_presence,
    textarea_scene,
    true,
);
```

This is useful for app-level panels that are compositional but not themselves separate machines.

## Pattern 2: participant-local collaboration cues

Use `PresenceCue` plus `presence_strip(...)` when you want to show participant-local presence without introducing shared-runtime complexity yet.

Example:

```rust
let cues = vec![
    PresenceCue {
        name: "you",
        label: "editing",
        tone: PresenceTone::Local,
    },
    PresenceCue {
        name: "mika",
        label: "reviewing",
        tone: PresenceTone::Collaborator,
    },
];

let strip = presence_strip(2_000_u64, &cues);
```

This is especially useful for:

- notes surfaces
- task/detail panes
- inspector-style metadata rows
- future participant-local overlays
- future header-local animation or presence effects

## Pattern 3: structured detail rows

Use `labeled_value(...)` for small metadata/detail layouts.

Example:

```rust
let priority = labeled_value(
    3_000_u64,
    "priority",
    "high".to_string(),
    Style::PLAIN.fg(Color::Ansi(1)).bold(),
);
```

This gives a cleaner result than hand-formatting a long text string like:

```text
priority: high
```

because label and value remain separately styleable scene nodes.

## Pattern 4: split layouts and shells

Two newer helpers support a first minimal application-layout layer:

- `split_columns(...)`
- `master_detail(...)`
- `app_shell(...)`
- `app_shell_with_theme(...)`
- `app_shell_with_fill(...)`

These are intentionally small, but they establish explicit contracts for:

- left/right workspace splits
- stacked list/detail regions inside a bounded slot
- application title/header/body/footer shells
- panel-local chrome and background treatment

They are an initial step toward the broader application-layout direction described in:

- [../architecture/06-application-layout-patterns.md](../architecture/06-application-layout-patterns.md)

## Pattern 5: bounded text helpers

`wrap_text_lines(...)` and `truncated_wrapped_lines(...)` are useful for detail panes that should:

- respect narrow widths
- avoid blowing up layout height
- degrade gracefully until a richer scrolling/text-display machine exists

Example:

```rust
let lines = truncated_wrapped_lines(detail_text, 24, 2);
let scene = Scene::column(
    4_000_u64,
    lines
        .into_iter()
        .enumerate()
        .map(|(index, line)| Scene::text(4_100_u64 + index as u64, line))
        .collect::<Vec<_>>(),
);
```

## When to use helpers vs machines

A useful rule of thumb:

### Prefer helper functions when

- the pattern is mostly presentational
- the pattern has no meaningful local state
- the pattern does not need its own message/update lifecycle
- the pattern is still evolving quickly inside one app

### Prefer a machine when

- the pattern has local state
- the pattern handles input/focus directly
- the pattern emits semantic messages
- the pattern is reusable across multiple apps as a real control

So:

- `surface_panel(...)` is a good helper
- `TextareaMachine` is a real machine

## Example: assembling a task detail panel

A typical usage pattern is:

```rust
let detail = Scene::column(
    5_000_u64,
    vec![
        labeled_value(5_001_u64, "state", "active".into(), active_style),
        labeled_value(5_002_u64, "priority", "high".into(), priority_style),
    ],
);

let panel = surface_panel(
    5_100_u64,
    28,
    "Tasks",
    "Focus to triage work",
    &task_presence,
    detail,
    false,
);
```

This is a good example of app-level composition that remains explicit and structural.

## Why this matters for Knopper

Knopper’s public abstraction is the machine, but not every reusable concept in an application needs to become a machine immediately.

The extracted `demo_ui` helpers demonstrate a practical layering strategy:

1. raw scene algebra
2. reusable scene-composition helpers
3. full reusable machines

That layering lets applications stay readable while keeping the machine abstraction focused on truly stateful semantic processes.

## Second demo usage

A second demo binary now uses these helpers in a different workspace shape:

```bash
cargo run --bin review_demo
```

That demo combines:

- a review queue surface
- a draft reply surface
- shared panel chrome via `surface_panel(...)`
- participant-local cues via `presence_strip(...)`
- structured detail rows via `labeled_value(...)`

This gives the extracted helper layer a second real consumer, which is useful for refining the patterns before promoting them further.

## Related guides

- [01-writing-a-machine.md](01-writing-a-machine.md)
- [02-composing-machines.md](02-composing-machines.md)
- [05-walkthrough-demo-workspace.md](05-walkthrough-demo-workspace.md)
- [06-running-the-interactive-demo.md](06-running-the-interactive-demo.md)
