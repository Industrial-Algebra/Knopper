# Scene Algebra

A `Scene<Msg>` is a declarative tree describing layout, content, interaction,
and focus structure. Machines project scenes; the runtime lays them out,
lowers them to render ops, and diffs them against the previous frame.

## Builders

```rust
Scene::text(id, "content")
Scene::row(id, children)        // horizontal flow
Scene::column(id, children)     // vertical flow
Scene::stack(id, children)      // same cell, back-to-front
Scene::overlay(id, children)    // floating above the base scene

Scene::padding(id, padding, child)
Scene::border(id, child)
Scene::sized(id, constraint, child)
Scene::viewport(id, child)      // clip to bounds
Scene::scroll(id, offset, child)
Scene::align(id, anchor, child)
Scene::annotated(id, label, child)

Scene::focus_scope(id, name, child)
Scene::focus_scope_with_policy(id, name, policy, child)
```

Every node takes a stable `NodeId`. **Stable ids are load-bearing**: the
renderer diffs by id, so an append that adds a new id produces an insert patch
without touching existing nodes (see
[Streaming-Append Performance](../design/streaming-append.md)).

## Modifiers

```rust
scene
    .with_style(style)        // fg/bg/emphasis
    .with_role(Role::Button)  // semantic role
    .focusable()              // participates in Tab order
    .disabled()               // skipped by focus + activation
    .on_activate(msg)         // Enter/click sends msg
    .map_msg(&f)              // translate child messages into parent space
```

`map_msg` is the composition primitive: a child machine's `Msg` type maps into
the parent's, so screens compose without a global message enum.

## Annotations and presence

`Annotation` carries out-of-band render metadata — including
`PresenceSlot` and `RemoteCursor`, the hooks for multi-participant presence
overlays. Annotations travel with the scene but don't affect layout; backends
render them as overlays.

## Focus scopes

`focus_scope` declares a named region with a policy:

- **`Wrap`** — Tab cycles within the scope
- **`Trap`** — Tab *clamps* at scope boundaries (modal semantics: focus cannot
  leave)
- **`Local`** — order is local to the scope, no cycling
- **`Passthrough`** — scope is transparent to traversal

See [The Focus Model](./focus-model.md) for how these interact with runtime
Tab handling.
