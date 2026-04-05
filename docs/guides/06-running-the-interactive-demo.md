# Running the Interactive Demo

## Purpose

This guide explains how to run and use Knopper’s current interactive demo host.

Knopper now has two demo interaction modes:

- a **raw-key interactive host**
- a **command-driven shell fallback**

Both host the composed demo workspace, apply real machine updates, and can optionally render through the Notcurses backend.

This makes the demo a practical development tool for validating composition, focus behavior, modal behavior, and rendering while the framework continues to mature.

## Run modes

## Raw-key interactive host

Run the default demo host with:

```bash
cargo run
```

This mode:

- runs in terminal raw mode
- redraws the workspace after each key event
- uses the current terminal size as the demo bounds
- updates bounds live on terminal resize
- presents an app-like workspace with Notes and Tasks surfaces
- supports a quick-actions palette for common workspace commands
- can show focus/cursor/input diagnostics when the inspector is enabled

## Notcurses-backed raw host

If you want each frame to also render through the Notcurses backend, run:

```bash
cargo run --features notcurses -- --notcurses
```

This mode keeps the raw-key host and also pushes rendered frames through Knopper’s Notcurses backend.

## Command shell fallback

If you want the older command-driven shell instead, run:

```bash
cargo run -- --shell
```

Important: this is not a fullscreen TUI. It is a diagnostic command driver that prints a command list and textual debug snapshots of the demo state.

## What the demo contains

The workspace includes:

- tabs
- a toggle
- a sync button
- a notes textarea surface
- a task list surface
- a quick-actions palette overlay
- a derived status line

This is the same demo described in:

- [05-walkthrough-demo-workspace.md](05-walkthrough-demo-workspace.md)

## Command overview

In raw-host mode, press `?` to toggle the short help hint.

In shell mode, run `help` to see the current command list.

Current commands are:

```text
show
help
quit
focus tabs|toggle|button|note|list|palette
tab
backtab
tabs left|right|commit|select <index>
toggle
sync
note text <text>
note nl
note backspace
note left|right|up|down
note commit
list up|down|commit
palette open|close
palette focus-input|focus-list
palette text <text>
palette backspace
palette up|down|commit
palette tab|backtab
```

## Raw-key controls

Current raw-key host behavior includes:

- `q` quits
- `?` toggles the help hint
- `Ctrl-P` opens the quick-actions palette
- `Ctrl-G` or `F2` toggles the inspector
- `Tab` / `Shift-Tab` traverse focus
- arrow keys drive focused controls where appropriate
- text entry inserts into the focused textarea or palette input
- `Enter` activates or commits depending on the focused control
- `Escape` exits the raw host when the palette is closed

## Shell-mode focus commands

### Focus a control directly

```text
focus tabs
focus toggle
focus button
focus note
focus list
focus palette
```

This sends a direct focus request to a meaningful node inside that control region.

### Move focus structurally

```text
tab
backtab
```

These use Knopper’s runtime focus traversal machinery:

- scene-derived focus order
- nearest-scope derivation
- focus scope policy application

So the shell mode is a useful way to exercise the real focus system.

## Shell-mode tabs commands

```text
tabs left
tabs right
tabs commit
tabs select 2
```

These drive the `TabsMachine` inside the demo.

## Shell-mode toggle and button commands

### Toggle shared mode

```text
toggle
```

### Press the sync button

```text
sync
```

These update child machine state and also affect the demo’s derived status line.

## Shell-mode textarea commands

### Insert text

```text
note text hello world
```

### Insert newline

```text
note nl
```

### Delete backward

```text
note backspace
```

### Move cursor

```text
note left
note right
note up
note down
```

### Commit the current note

```text
note commit
```

This demonstrates the local-first textarea machine and the parent’s derived status behavior. Pressing the sync button also snapshots the current note into the saved status state so the footer reflects a more lifelike workspace flow.

## Shell-mode list commands

```text
list up
list down
list commit
```

These drive the task list selection and activation semantics. The footer summarizes the currently selected task by name rather than by raw index.

## A quick sample session

Example:

```text
focus note
note text Knopper
note nl
note text demo
note commit
toggle
sync
tabs select 1
show
```

This will produce a workspace state roughly like:

- textarea note committed as `Knopper | demo`
- shared mode enabled
- sync count incremented
- `Notes` tab selected

## What is displayed after each frame or command

The raw host and shell both currently print:

- the workspace snapshot
- the current `DemoState`
- the current local `FocusState`
- the current cursor position

This is useful for understanding how updates propagate through:

- child machines
- parent-derived state
- focus traversal
- cursor placement
- scene rendering

## Command palette commands

### Open or close the palette

```text
palette open
palette close
```

### Move focus inside the palette

```text
palette focus-input
palette focus-list
palette tab
palette backtab
```

### Edit the palette query

```text
palette text sync
palette backspace
```

### Navigate and commit palette items

```text
palette up
palette down
palette commit
```

This is a useful way to exercise modal and focus-scope behavior in the current demo host.

## What this demo is good for

The interactive demo host is especially useful for:

- validating machine composition
- checking focus traversal and scope behavior
- testing textarea and list behavior quickly
- exercising command palette modal behavior
- observing parent-derived state updates
- validating Notcurses rendering integration in a lightweight way

## What it is not yet

The current host is not yet:

- the final full-screen application runtime architecture for Knopper
- a complete backend-neutral event host abstraction
- the final polished user-facing demo experience

It is best understood as a practical bridge between:

- pure unit-tested framework behavior
- and a future richer interactive runtime integration

## Suggested experiments

Try these to pressure-test the framework.

### Focus traversal

```text
focus tabs
tab
tab
backtab
```

### Text editing

```text
focus note
note text multi line
note nl
note text editor
note up
note right
note commit
```

### Combined workspace semantics

```text
toggle
sync
tabs select 2
list down
list commit
show
```

Watch how the derived status line changes as child state changes.

## Bottom line

If you remember one sentence from this guide, remember this:

> the interactive demo host is now the main hands-on way to exercise Knopper’s real machine, focus, modal, composition, and rendering pipeline, while shell mode remains a useful scripted fallback.
