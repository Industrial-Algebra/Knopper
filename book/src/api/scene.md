# Scene Builders

`Scene<Msg>` nodes and modifiers. Every builder takes a stable id
(`impl Into<NodeId>`); see [Scene Algebra](../concepts/scene-algebra.md) for
the design rationale.

## Structure

| Builder | Layout |
| ------- | ------ |
| `text(id, content)` | leaf text node |
| `row(id, children)` | horizontal flow |
| `column(id, children)` | vertical flow |
| `stack(id, children)` | same cell, back-to-front |
| `overlay(id, children)` | floating above the base |

## Wrappers

| Builder | Effect |
| ------- | ------ |
| `padding(id, Padding, child)` | inset space |
| `border(id, child)` | drawn border |
| `sized(id, SizeConstraint, child)` | width/height constraint |
| `viewport(id, child)` | clip to bounds |
| `scroll(id, ScrollOffset, child)` | scrolled content |
| `align(id, Anchor, child)` | anchor within available space |
| `annotated(id, label, child)` | attach an annotation label |

## Focus

| Builder | Effect |
| ------- | ------ |
| `focus_scope(id, name, child)` | named scope, default policy |
| `focus_scope_with_policy(id, name, policy, child)` | `Wrap` / `Trap` / `Local` / `Passthrough` |

## Modifiers (chainable)

```rust
.with_style(Style)            // Color/Emphasis
.with_role(Role)              // semantic role (Button, etc.)
.with_annotation(Annotation)  // presence/cursor overlays, labels
.focusable()                  // enter the Tab order
.disabled()                   // skip focus + activation
.on_activate(msg)             // Enter/click → msg
.map_msg(&fn)                 // Msg → ParentMsg translation
```

## Supporting types

`Style`, `Color`, `Emphasis` · `Role` · `Annotation` · `Padding` ·
`SizeConstraint` / `Size` · `ScrollOffset` · `Anchor` · `NodeId`

All are plain data — scenes are cheap to construct per projection, and the
id-keyed diff keeps commits cheap when ids are stable.
