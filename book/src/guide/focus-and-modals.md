# Focus, Modals, and Disabled Nodes

## Modals: declarative trapping

A modal is two declarations:

```rust
// 1. The modal subtree is a Trap focus scope.
Scene::focus_scope_with_policy(
    modal_id,
    "confirm-dialog",
    FocusScopePolicy::Trap,
    dialog_scene,
)

// 2. Escape (or a close action) sends a close message.
```

`Trap` makes Tab clamp at the scope boundary — focus physically cannot leave
the modal via traversal. No imperative Tab interception, no key filtering in
the modal's `key_msg`. When the modal closes, a refocus safety net returns
focus to a sensible node (the runtime and the `compose::trap_focus` helper
coordinate this).

This replaced an earlier imperative design where modals handled Tab in their
own key path. The migration is described in `docs/roadmap/00` (M1) and the
semantics are pinned by end-to-end tests (`trap_scope_clamps_focus_within_scope_under_runtime_tab`).

## Disabled nodes

```rust
Scene::text(id, "delete").focusable().disabled()
```

A `.disabled()` node:

- is **skipped by focus collection** — Tab never lands on it
- is **skipped by activation routing** — Enter/click on it sends nothing

It still renders (style it dim via `with_style`), so "present but
unavailable" is a first-class state. Toggling `.disabled()` per-frame from
`Model` is the intended way to express dynamic availability — e.g. a submit
button disabled while a form is invalid.

## Eligibility rules, precisely

Focus collection includes a node iff: it is `.focusable()` **and** not
`.disabled()` **and** inside the visible scene. Scope policies then shape how
traversal moves within that collected order. `Effect::RequestFocus` to a
disabled or absent node is ignored.
