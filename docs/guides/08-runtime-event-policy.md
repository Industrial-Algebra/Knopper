# Knopper runtime event policy

> **Guide** — how events flow through a Knopper `Runtime`, and the dispatch
> precedence rules every host and parent machine should follow.
>
// Companion to the architecture docs in `docs/architecture/`.

## Why this matters

Knopper does not prescribe a single global key handler. The `Runtime`
handles a small set of **structural** events declaratively (focus
movement, activation), while **semantic** keys (arrows, letters, Escape,
function keys) are interpreted by machines. Getting the precedence right
is what keeps composed machines cooperating instead of fighting over keys.

This document fixes the precedence rules so hosts and parent machines
behave consistently, and so downstream projects know exactly where to hook
in.

## The event flow

```
RuntimeEvent
     │
     ▼
route_event(scene, &mut focus, event)   ── src/routing.rs
     │
     ├─ Activate(id)  → activation_message(scene, id)  → Message(msg) | Ignored
     ├─ Focus(id)     → focus_path(scene, id)          → FocusChanged   | Ignored
     ├─ Key(Enter)    → activate focused node          → Message(msg)   | Ignored
     ├─ Key(Tab)      → FocusNavigation::advance       → FocusChanged   | Ignored
     └─ everything else → Ignored (deferred to machines)
     │
     ▼
Runtime::dispatch applies the routed result:
     ├─ Message(msg) → machine.update(model, msg, ctx) → Effect
     └─ FocusChanged → updates runtime focus, no machine update
```

The key insight: **the runtime's `route_event` only handles structural
events.** Semantic keys (anything that is not Enter or Tab) return `Ignored`
from `route_event` and must be interpreted by a machine via its own
`key_msg` helper (or the host's key handler).

## Dispatch precedence (the canonical order)

Hosts and parent machines should apply keys in this order. This is the
pattern the in-tree demos (`src/main.rs`) follow:

1. **Application-global shortcuts.** Host-level chords that frame the whole
   app (e.g. `Ctrl-G` toggles an inspector, `Ctrl-P` opens a palette).
   These never reach the runtime or machines.

2. **Declarative focus keys.** `Tab` and `Shift-Tab` go to
   `Runtime::dispatch(RuntimeEvent::Key(...))` so the runtime's
   `FocusNavigation` owns focus movement against the scene's focus scopes.
   Do **not** handle Tab in machine `key_msg` — that is what created the
   old imperative/declarative duplication (now removed; see
   [focus-and-modal-semantics](guides/03-focus-and-modal-semantics.md)).

3. **Machine-specific key handling.** Everything else is offered to the
   focused machine's `key_msg` first:
   ```text
   if let Some(msg) = machine.key_msg(&focus, &model, key, ctx) {
       runtime.send(msg);
   } else {
       runtime.dispatch(RuntimeEvent::Key(key));  // structural fallback
   }
   ```
   The machine gets the first say because it knows its own semantics
   (arrows move a list cursor, letters insert into an input, Escape
   dismisses a modal).

4. **Declarative fallback.** If the machine declines the key (`key_msg`
   returns `None`), it goes to `Runtime::dispatch`, which currently only
   handles Enter/Tab structurally. Most non-structural keys are simply
   dropped here.

## Parent/child dispatch rules

Parent machines compose child machines. The rules for routing a key to the
right child:

- **Dispatch by focus.** Use `child_has_focus(focus, child_root)` to decide
  which child receives the key. Only one child should interpret a given key
  press. The `compose` module provides `dispatch_if_focused` and
  `child_has_focus` for this.

- **Lift child messages.** A child's `key_msg` produces a child message;
  the parent lifts it into its own message space
  (`CommandPaletteMsg::Input(InputMsg::...)`). The `project_child` and
  `update_child` helpers in `compose.rs` handle the analogous lift for
  projection and update.

- **Modal trapping is structural.** When a modal is open, its
  `FocusScopePolicy::Trap` scope (declared in the scene) keeps focus inside
  it via the runtime's Tab handling. The imperative `modal_key_msg` only
  handles Escape and the focus-refocus safety net — it no longer handles
  Tab. See `src/standard/modal.rs`.

- **Do not steal focus from children for non-focus concerns.** If a parent
  needs to react to a key, it should do so before delegating to children,
  and only for true parent-level concerns (palette open/close, tab
  switching). Otherwise the focused child wins.

## Focus and cursor synchronization

- **Focus is runtime-owned and participant-local.** `Runtime::focus()`
  returns the single local `FocusState`. There is no global focus. See
  [06-collaboration-ready-contract](roadmap/06-collaboration-ready-contract.md)
  §1 for the multi-participant model.

- **Activation follows focus.** `Enter` activates the currently focused
  node by looking up its `on_activate` message via `activation_message`.
  A node with no `Interaction::Activate` produces `Ignored`.

- **The cursor follows the focused control.** `Machine::cursor_position`
  returns the local participant's cursor position based on model + layout.
  `Runtime::cursor(bounds)` clamps it to the bounds. Hosts call
  `render_to_backend_auto_cursor` to emit both the diff and the cursor in
  one pass. Remote cursors are not handled here — they are presence
  overlays derived from `Shared` during projection.

- **Disabled nodes are inert.** A `disabled` node is skipped by focus
  collection and returns no activation message, so the runtime will never
  route a key to it. See `NodeMeta::disabled` / `Scene::disabled()`.

## What the runtime does NOT do

- It does not interpret semantic keys (arrows, letters, Escape, F-keys).
  That is machine territory.
- It does not bubble events. Dispatch is one-shot: route → update → effect.
  Parent/child composition is the parent machine's responsibility.
- It does not own collaboration. Shared-state updates arrive via
  `set_shared`; focus stays local.
- It does not handle capture/preview phases. If a future need arises, it
  will be added explicitly, not retrofitted silently.

## Checklist for new hosts and parent machines

- [ ] Global shortcuts handled before any dispatch.
- [ ] `Tab` / `Shift-Tab` routed only through `Runtime::dispatch`.
- [ ] Semantic keys offered to the focused machine's `key_msg`, with
      `Runtime::dispatch` as the fallback.
- [ ] Child dispatch decided by `child_has_focus` / `dispatch_if_focused`.
- [ ] Modal trapping relies on `FocusScopePolicy::Trap`, not imperative Tab.
- [ ] Cursor emitted via `render_to_backend_auto_cursor` (or equivalent).
- [ ] Disabled nodes never receive keys.

---

*Guide for Knopper 0.1.0. See `tests/runtime_pipeline.rs` for executable
characterizations of this policy.*
