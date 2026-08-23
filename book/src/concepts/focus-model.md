# The Focus Model

Focus in Knopper is **scene-derived, scope-aware, and participant-local**.

## Scene-derived order

The focus order is collected by walking the projected scene: nodes marked
`.focusable()` enter the order in scene position. There is no separately
maintained focus list — change the scene and the focus order follows.
`.disabled()` nodes are skipped by both focus collection and activation
routing, so disabling a control is a presentation-free state change.

## Runtime-owned Tab

Tab / Shift-Tab are owned by the **runtime's declarative dispatch**, not by
individual machines. The dispatch precedence is:

1. Global shortcuts (host/machine-declared)
2. Declarative Tab traversal across the scene-derived focus order
3. The focused machine's `key_msg` handler
4. Declarative fallback (activation, focus events)

Machines opt into raw keys via `key_msg`; everything else flows through the
declarative path. The full policy is specified in the repo at
`docs/guides/08-runtime-event-policy.md`.

## Scope policies

Focus scopes (`Scene::focus_scope_with_policy`) control how Tab behaves at
boundaries:

| Policy | Tab at last node | Use |
| ------ | ---------------- | --- |
| `Wrap` | Cycles to the scope's first node | toolbars, tab bars |
| `Trap` | **Clamps — focus stays put** | modals, dialogs |
| `Local` | Stops; no cycling | grouped controls |
| `Passthrough` | Scope is transparent | layout-only regions |

**Trap clamps; it never wraps.** A modal is a `Trap` scope plus an Escape
message — the declarative policy does the containment, so modal
implementations need no imperative Tab handling.

## Participant-local by design

Each `Runtime` owns its own `FocusState`. In a multi-participant session every
participant has an independent focus — two runtimes over the same `Shared`
projection diverge in focus without cross-talk. Focus is never pushed through
`Shared` and there is no "global focus" concept. Remote participants' cursors
and selections appear as `Annotation` overlays, not focus.

`Effect::RequestFocus(NodeId)` lets a machine request focus movement; the
runtime applies it through the same dispatch path as user input.
