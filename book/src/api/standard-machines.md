# Standard Machines

Ready-made machines in `knopper::standard`, each exporting its `*Machine`,
`*Msg`, `*State`, `*Context`, and a `*_key_msg` raw-key handler. Compose them
with the helpers in [`knopper::compose`](../guide/composing-machines.md).

| Machine | Purpose | Notes |
| ------- | ------- | ----- |
| `ButtonMachine` | activation control | `on_activate` semantics |
| `ToggleMachine` | boolean switch | state in `ToggleState` |
| `InputMachine` | single-line text input | participant-local buffer |
| `TextareaMachine` | multi-line editing | see roadmap doc 04 for the editor design |
| `ListMachine` | selectable list | selection/commit in `Model` — multi-user-safe |
| `ListDetailMachine` | list + detail pane | two-region navigation |
| `TabsMachine` | tab bar | `Wrap`-policy scope |
| `CommandPaletteMachine` | fuzzy command overlay | input + filtered list composition |
| modal helpers | declarative dialog trapping | `ModalFocusConfig`, `ModalKeyAction`; see [modals](../guide/focus-and-modals.md) |

## Composition contract

Each standard machine:

1. Keeps interaction state in its `*State` (participant-local `Model` lane)
2. Reads display data from `Shared` where the data is projection-shaped
   (e.g. list items)
3. Exposes `*_key_msg` for its raw-key behavior, wired into the host's or
   parent's `key_msg`/dispatch as needed
4. Never hardcodes single-user ownership — no global focus, no process
   singletons

The collaboration audit (`docs/roadmap/02-standard-machine-collaboration-audit.md`)
verifies each machine against these rules.

## Example: palette in a parent

The demo (`src/demo.rs`) composes the command palette inside the workspace
machine: palette state lives in the parent's `Model`, palette messages lift
through `map_msg`, and the palette's scene subtree sits in a `Trap` scope
while open. Read it alongside
[Composing Machines](../guide/composing-machines.md).
