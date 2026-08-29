# Composing Machines

Screens are built by composing standard machines inside a parent machine, using
the helpers in `knopper::compose`.

## The composition vocabulary

- **`update_child`** — route a parent's `Msg` variant into a child machine's
  `update`, mapping the child's `Effect` back into parent space
- **`project_child`** — embed a child's projected scene in the parent's scene,
  mapping messages with `map_msg`
- **`map_effect`** — lift an `Effect<ChildMsg>` to `Effect<ParentMsg>`
- **`child_has_focus`** — is focus anywhere under this child's scene root?
- **`dispatch_if_focused`** — forward a key to a child only when it holds focus
- **`trap_focus`** — keep focus inside a scope (modal support)

## Pattern

```rust
// Parent Model holds child states; parent Msg wraps child msgs.
enum Msg { Palette(PaletteMsg), List(ListMsg) }

// update: route by variant
Msg::Palette(inner) => update_child(&mut model.palette, inner, &ctx.palette),

// project: embed with map_msg
let palette_scene = project_child(&palette_machine, &model.palette, &shared, &ctx.palette)
    .map_msg(&Msg::Palette);
```

Each standard machine exports its `*Context` / `*Msg` / `*State` types and a
`*_key_msg` handler for its raw-key behavior — see
[Standard Machines](../api/standard-machines.md).

## Focus across composition

Child scenes participate in the parent's scene-derived focus order. Wrap a
child subtree in `Scene::focus_scope` to give it its own traversal policy —
that's how the demo's modal palette traps focus without imperative key
routing (see [Focus, Modals, and Disabled Nodes](./focus-and-modals.md)).

## Reference

`docs/guides/07-reusable-composition-patterns.md` in the repo catalogs the
extracted patterns with worked code from the demo applications.
