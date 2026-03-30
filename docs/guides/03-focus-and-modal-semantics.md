# Focus and Modal Semantics in Knopper

## Purpose

This guide explains how focus traversal, focus scopes, and modal behavior work in Knopper.

It is especially important because Knopper treats focus as a structural semantic concern rather than a hidden widget implementation detail.

This guide also explains the collaboration-aware framing of focus:

> local focus is participant-local by default, even in future collaborative applications.

## The mental model

Knopper focus is built from a few composable ideas:

- **focusable scene nodes**
- **focus paths**
- **focus order derived from scene structure**
- **focus scopes with explicit policies**
- **runtime traversal using `Tab` / `Shift-Tab`**
- **modal wrappers that trap or redirect focus structurally**

The key point is that focus is not meant to be hand-coded independently inside every control.

Instead, focus behavior should mostly emerge from:

- scene structure
- stable node identity
- focus annotations on scene nodes
- explicit scope policy when local traversal regions exist

## Focus is participant-local

This is the most important semantic rule.

Knopper’s current focus model should be understood as local to one runtime instance.

That means:

- each runtime has its own `FocusState`
- local keyboard traversal affects local focus only
- remote participant activity should not overwrite local focus
- collaborative presence should later appear as overlays, not focus theft

So when this guide says “focus,” it means:

- the local participant’s focus inside their runtime

This is intentional and collaboration-ready.

## The core types

## `FocusState`

`FocusState` stores the current local focus path.

Conceptually:

- it is owned by the runtime
- it is updated by focus events and keyboard traversal
- it is not shared semantic truth for all participants

## `FocusPath`

A `FocusPath` is the path from the scene root to the currently focused node.

This is useful because it preserves structure, not just a single focused ID.

That allows helpers such as:

- subtree focus checks
- child focus routing
- nearest-scope derivation

## `FocusOrder`

A `FocusOrder` is an ordered collection of focusable node IDs.

Knopper can derive one from:

- the whole scene
- a named focus scope
- the nearest scope around the current focus

This is the basis for runtime traversal.

## `FocusNavigation`

`FocusNavigation` combines:

- a `FocusOrder`
- a `FocusScopePolicy`

This is what the runtime uses to decide how `Tab` and `Shift-Tab` should behave from the current focus location.

## Focusable nodes

A scene node participates in focus traversal when it is marked focusable.

Example:

```rust
Scene::text(id, "run").focusable()
```

Activation often also implies focusability for text nodes:

```rust
Scene::text(id, "run").on_activate(MyMsg::Run)
```

If a node is not focusable, it will not appear in derived focus order.

## Focus order is scene-derived

Knopper derives focus traversal order from scene structure rather than requiring every control to manually maintain its own tab graph.

Important helper methods include:

- `FocusOrder::collect_from_scene(...)`
- `FocusOrder::collect_from_scope(...)`
- `FocusOrder::collect_from_nearest_scope(...)`
- `FocusOrder::collect_from_focus_path(...)`
- `FocusOrder::collect_for_focus(...)`

This means tab traversal is usually a property of:

- the scene tree
- the current focus position
- the nearest enclosing scope policy

## Focus scopes

A focus scope defines a local traversal region in the scene tree.

You create one with:

```rust
Scene::focus_scope(scope_id, "name", body)
```

or explicitly with policy:

```rust
Scene::focus_scope_with_policy(
    scope_id,
    "name",
    knopper::FocusScopePolicy::Trap,
    body,
)
```

A focus scope is structural and semantic:

- it affects focus derivation and traversal semantics
- it does not change rendering by itself

## Focus scope policies

Knopper currently supports four scope policies.

## `Wrap`

Traversal stays within the nearest scope and wraps at the ends.

Meaning:

- `Tab` from the last focusable item cycles back to the first
- `Shift-Tab` from the first cycles to the last

Use this when the scope is a self-contained local traversal ring.

## `Trap`

Traversal behaves like wrap locally, but semantically indicates that focus should remain trapped inside the scope.

Use this for modal-like regions and local interaction islands.

In practice, this is the right default for many modal bodies.

## `Local`

Traversal is limited to the nearest scope but does **not** wrap.

Meaning:

- `Tab` at the end stops
- `Shift-Tab` at the beginning stops

Use this when the scope is local but should not cycle automatically.

## `Passthrough`

Traversal prefers the nearest scope first, but can fall back to the whole-scene order at the scope boundary.

Meaning:

- move locally while there is another focusable node in the scope
- once the boundary is reached, continue in the wider scene order

Use this for larger app sections where local structure matters but should not fully isolate traversal.

## Runtime focus traversal

The runtime handles generic keyboard focus traversal.

Current built-in behavior includes:

- `Enter` activates the currently focused activatable node
- `Tab` moves forward
- `Shift-Tab` moves backward

Traversal works by:

1. deriving whole-scene order
2. deriving nearest-scope navigation from current focus
3. applying the active scope policy
4. resolving the next node into a `FocusPath`
5. updating `FocusState`

This lives in the routing layer rather than inside each control.

## Direct focus events

The runtime also supports direct focus requests through:

- `RuntimeEvent::Focus(node_id)`
- `Effect::RequestFocus(node_id)`

This is useful when a machine wants to explicitly move focus after an update.

Examples:

- list selection requests focus on the selected marker
- tabs request focus on the selected tab
- button/toggle may request focus back to their root

## Focus-aware composition helpers

Knopper provides several helpers for composed machines.

## `child_has_focus(...)`

Checks whether the current focus path lies within a child subtree.

Use this when a parent needs to decide which child should receive keyboard input.

Example shape:

```rust
if child_has_focus(focus, self.input_root_id(ctx)) {
    // send key to input child
}
```

## `dispatch_if_focused(...)`

Conditionally returns a child message only if the given child currently has focus.

This is a compact way to gate child key dispatch.

## `trap_focus(...)`

Returns a fallback node when focus is outside a scope that should own focus locally.

This is useful in modal behavior where local focus should be redirected back into the modal.

## `next_focus_in_order(...)` / `previous_focus_in_order(...)`

These helpers move within an explicit `FocusOrder`.

They are useful in modal helpers and other places where the parent already knows the intended local order.

## Modal semantics

A modal in Knopper is not just a visual overlay.

It is a structural interaction region with:

- a backdrop
- a dialog body
- local focus behavior
- dismissal semantics

The reusable modal helper lives in `src/standard/modal.rs`.

Important pieces include:

- `ModalIds`
- `ModalMsg<Msg>`
- `ModalFocusConfig<Msg>`
- `modal_key_msg(...)`
- `modal_scene(...)`

## `modal_scene(...)`

This helper wraps a body scene into modal structure.

It creates:

- overlay root
- backdrop layer
- centered dialog body

The backdrop is focusable and can emit dismiss behavior through activation.

## `modal_key_msg(...)`

This helper handles modal keyboard semantics such as:

- `Escape` to dismiss
- `Tab` / `Shift-Tab` within modal order
- re-focusing the modal’s primary target if local focus escapes the scope

This lets modal behavior remain reusable instead of being reimplemented inside every modal-like machine.

## Example: command palette modal behavior

`CommandPaletteMachine` is a strong reference for focus and modal semantics working together.

It composes:

- input child
- list child
- modal helper behavior
- a trapped internal focus scope

Important patterns it demonstrates:

- focus order derived from the composed modal body
- local modal focus configuration
- child key dispatch based on local focus
- backdrop dismissal
- structural modal composition rather than hardcoded overlay logic

## Choosing a scope policy

A useful rule of thumb:

### Use `Trap` when:

- the region is modal-like
- focus should not escape casually
- you want a local focus island

### Use `Wrap` when:

- the region is self-contained
- cycling is desirable
- wrapping is part of the control’s expected behavior

### Use `Local` when:

- the region is local
- you do not want wrapping
- stopping at the boundary is correct

### Use `Passthrough` when:

- the region is meaningful but not isolated
- local structure should influence traversal
- focus should still continue into the broader app when appropriate

## Focus scopes in app-level composition

At the application level, focus scopes are often useful for:

- modal bodies
- local tool palettes
- sidebars
- editor panes
- popovers or overlays
- structured regions with intentional traversal semantics

You do not need a focus scope everywhere.

Add one when a subtree has meaningful local traversal rules.

## Stable identity matters

Focus depends on stable node identity.

When authoring machines, keep IDs stable for:

- roots
- focusable children
- semantically persistent surfaces

Stable IDs help with:

- focus path resolution
- request-focus effects
- predictable tests
- future collaboration overlays

## Focus and collaboration

Knopper is designed so focus remains participant-local even as collaboration deepens.

That means future collaborative behavior should prefer:

- local `FocusState` per participant runtime
- remote presence overlays for remote attention/cursor-like state
- shared semantic state for the underlying application model

Do **not** treat focus as the same thing as:

- shared selection truth
- remote cursor publication
- globally authoritative UI ownership

This is one of the key ways Knopper avoids drifting into a single-global-user model.

## Testing focus behavior

Useful focus tests include:

### Traversal tests

Verify `Tab` / `Shift-Tab` behavior in normal scenes and scoped scenes.

### Scope policy tests

Verify wrap, trap, local, and passthrough behavior explicitly.

### Modal tests

Verify:

- escape dismissal
- backdrop dismissal
- focus redirection into the modal
- local traversal order

### Composition tests

Verify that only the focused child receives key-driven child dispatch where appropriate.

The existing routing, modal, command palette, and focus tests are good references.

## Common mistakes to avoid

### 1. Hardcoding focus movement inside every control

Prefer scene focusability, focus scopes, and shared traversal helpers.

### 2. Using modal behavior without structural scope/focus rules

A visible overlay is not enough; modals also need local interaction semantics.

### 3. Treating focus as shared collaborative truth

Focus should remain local unless you are explicitly modeling published participant presence.

### 4. Choosing unstable IDs for focusable nodes

Focus paths and request-focus effects depend on stable semantic anchors.

### 5. Forgetting boundary behavior

As soon as a subtree has multiple focusable nodes, be explicit about whether it wants:

- wrap
- trap
- local stop
- passthrough

## Recommended code references

Useful files to study alongside this guide:

- `src/focus.rs`
- `src/routing.rs`
- `src/standard/modal.rs`
- `src/standard/command_palette.rs`
- `src/compose.rs`
- `docs/roadmap/01-collaboration-readiness.md`

## Bottom line

If you remember one sentence from this guide, remember this:

> in Knopper, focus and modal behavior should be expressed structurally through focusable scene nodes, scope policies, and reusable runtime/modal helpers rather than hidden per-widget tricks.
