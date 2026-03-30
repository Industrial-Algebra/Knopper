# Composing Machines in Knopper

## Purpose

This guide explains how to build larger Knopper interfaces by composing machines together.

If [writing a machine](01-writing-a-machine.md) explains the unit of abstraction, this guide explains how those units fit together.

In practice, composition is one of Knopper’s biggest strengths:

- small controls remain real machines
- parent machines can own additional state and semantics
- child scenes and effects can be lifted structurally
- focus and modal behavior can remain explicit and reusable

## The mental model

A composed machine is still just a machine.

There is no separate “container component” model.

A parent machine typically does three things:

1. stores child model state inside its own `Model`
2. lifts child messages into parent messages
3. projects child scenes into a larger parent scene

The important point is:

> composition in Knopper is ordinary machine authoring, not a separate subsystem.

## Core helpers

The main composition helpers live in `src/compose.rs`.

They are:

- `project_child(...)`
- `update_child(...)`
- `map_effect(...)`
- `child_has_focus(...)`
- `dispatch_if_focused(...)`
- `trap_focus(...)`
- `next_focus_in_order(...)`
- `previous_focus_in_order(...)`

Most composed machines use only a subset of these, but together they form the basic toolkit for nested machine authoring.

## The three main composition problems

When composing child machines, you usually need to solve three problems.

### 1. How do I store child state?

Put child model state inside the parent model.

Example:

```rust
pub struct ParentState {
    pub input: InputState,
    pub list: ListState,
    pub committed: Option<usize>,
}
```

### 2. How do I lift child messages?

Wrap child messages in parent message variants.

Example:

```rust
pub enum ParentMsg {
    Input(InputMsg),
    List(ListMsg<()>),
    Dismiss,
}
```

### 3. How do I project child scenes into parent space?

Use `project_child(...)` and give it a message-lifting function.

Example:

```rust
let input_scene = project_child(
    &self.input,
    &model.input,
    &(),
    &ctx.input,
    &ParentMsg::Input,
);
```

## `project_child(...)`

This helper projects a child machine into a parent message space.

Signature shape:

```rust
project_child(machine, model, shared, ctx, lift)
```

What it does:

- calls the child machine’s projection
- maps child messages into parent messages
- returns a `Scene<ParentMsg>`

That means child machines remain fully reusable.

### Example

```rust
let list_scene = project_child(
    &self.list,
    &model.list,
    &(),
    &ctx.list,
    &ParentMsg::List,
);
```

## `update_child(...)`

This helper updates a child machine while lifting any child effects into parent effects.

Signature shape:

```rust
update_child(machine, model, msg, ctx, lift)
```

What it does:

- runs the child’s `update(...)`
- rewrites emitted child messages into parent messages
- preserves non-message effects like `RequestFocus`

### Example

```rust
let effect = update_child(
    &self.list,
    &mut model.list,
    list_msg,
    &ctx.list,
    &ParentMsg::List,
);
```

This is usually the easiest correct way to update nested machines.

## `map_effect(...)`

`update_child(...)` is built on top of `map_effect(...)`.

Use `map_effect(...)` directly when you already have a child `Effect<ChildMsg>` and want to rewrite it into an `Effect<ParentMsg>`.

This is useful if the parent needs to do custom child-update logic before lifting effects.

## The standard parent-message pattern

A common pattern is:

```rust
pub enum ParentMsg {
    Child(ChildMsg),
    ParentOnly,
}
```

Then update logic becomes:

```rust
match msg {
    ParentMsg::Child(child_msg) => update_child(
        &self.child,
        &mut model.child,
        child_msg,
        &ctx.child,
        &ParentMsg::Child,
    ),
    ParentMsg::ParentOnly => Effect::None,
}
```

This pattern is simple, explicit, and scales well.

## Example: composition in `ListDetailMachine`

`src/standard/list_detail.rs` is a good minimal composition example.

Its parent model stores:

- child `list: ListState`
- parent-local `committed: Option<usize>`

Its messages are:

- `List(ListMsg<RowMsg>)`
- `Key(KeyEvent)`

Its update path:

- optionally derives parent-local meaning from child messages
- delegates child state update with `update_child(...)`

This is an important composition pattern:

> parent machines can layer additional semantics on top of child machines without breaking child reuse.

For example, `ListDetailMachine` treats list commit as “show detail for this item” while still using `ListMachine` as-is.

## Example: composition in `CommandPaletteMachine`

`src/standard/command_palette.rs` is a richer example.

It composes:

- `InputMachine`
- `ListMachine`
- modal helper behavior
- local parent state such as filtering and committed selection mapping

This shows that a parent machine can:

- own multiple children
- transform child context
- derive additional parent-local state
- manage focus between children
- wrap children in modal/focus-scoped scene structure

This is very close to the intended Knopper style for real applications.

## Parent-local state vs child state

One of the most important design choices in composition is deciding what belongs where.

### Child state belongs in the child when:

- it is intrinsic to the child control
- the child should remain reusable across many parents
- the state has a well-defined local meaning inside the child

Examples:

- input cursor state inside `InputMachine`
- list scroll/selected state inside `ListMachine`
- tabs selected state inside `TabsMachine`

### Parent-local state belongs in the parent when:

- it depends on how the parent interprets child output
- it coordinates multiple children
- it derives higher-level app behavior

Examples:

- command palette filtered indices
- command palette committed original-item mapping
- demo workspace derived status line
- list-detail committed selection meaning

## Composition with multiple children

A parent with multiple child machines often has a model like:

```rust
pub struct ParentState {
    pub input: InputState,
    pub list: ListState,
    pub button: ButtonState,
    pub status: String,
}
```

And messages like:

```rust
pub enum ParentMsg {
    Input(InputMsg),
    List(ListMsg<()>),
    Button(ButtonMsg),
}
```

Then `project_once(...)` can build the full composed body from child projections.

Example shape:

```rust
let input = project_child(...);
let list = project_child(...);
let button = project_child(...);

Scene::column(root_id, vec![input, list, button])
```

## Focus-aware composition

When multiple children exist, the parent often decides which child should receive keyboard input.

Useful helpers:

- `child_has_focus(...)`
- `dispatch_if_focused(...)`

### `child_has_focus(...)`

Checks whether current focus lies within a child subtree.

Example:

```rust
if child_has_focus(focus, self.input_root_id(ctx)) {
    // route key to input child
}
```

### `dispatch_if_focused(...)`

Conditionally returns a child message only when that child currently has focus.

Example:

```rust
dispatch_if_focused(
    focus,
    self.list.root_id(),
    self.list.key_msg(&state.list, event).map(ParentMsg::List),
)
```

This is especially helpful in composed controls that route raw keys differently depending on the focused child.

## Modal and scoped composition

Sometimes the parent needs to create a local interaction region for composed children.

This is where focus scopes and modal helpers matter.

For example, the command palette wraps its inner body with:

- a focus scope
- modal scene structure
- modal focus config

That gives it:

- local traversal policy
- focus trapping semantics
- reusable modal behavior

The important design lesson is:

> composition is not only about nesting scenes; it is also about defining interaction structure.

## Using focus scopes in composed machines

If your parent creates a meaningful local traversal region, wrap the relevant subtree:

```rust
Scene::focus_scope_with_policy(
    scope_id,
    "my-scope",
    knopper::FocusScopePolicy::Trap,
    body,
)
```

Choose policy based on semantics:

- `Wrap`
- `Trap`
- `Local`
- `Passthrough`

A modal-like region often wants `Trap`.
A larger app section may want `Passthrough`.

## Context transformation

A parent can pass transformed context to a child.

This is very common in composition.

Examples:

- filtered list items passed into a list child
- child width/height adjusted by parent layout policy
- parent-generated node IDs passed to a child context

The command palette does this by creating a filtered list context before projecting or updating the list child.

This is a key part of Knopper composition:

> parent machines can adapt child context without changing child implementation.

## A typical composition flow

Most composed machines follow this rough pattern.

### Step 1: define child machines in the parent struct

```rust
pub struct ParentMachine {
    input: InputMachine,
    list: ListMachine<...>,
}
```

### Step 2: store child state in the parent model

```rust
pub struct ParentState {
    pub input: InputState,
    pub list: ListState,
}
```

### Step 3: wrap child messages in parent messages

```rust
pub enum ParentMsg {
    Input(InputMsg),
    List(ListMsg<()>),
}
```

### Step 4: delegate updates with `update_child(...)`

```rust
match msg {
    ParentMsg::Input(msg) => update_child(...),
    ParentMsg::List(msg) => update_child(...),
}
```

### Step 5: project children with `project_child(...)`

```rust
let input = project_child(...);
let list = project_child(...);
```

### Step 6: assemble parent scene

```rust
Scene::column(root_id, vec![input, list, status])
```

## Deriving parent state from child messages

Parents often want to do more than blindly forward child updates.

For example, before or after calling `update_child(...)`, the parent can inspect the child message and derive some parent-local consequence.

Example shape:

```rust
match msg {
    ParentMsg::List(list_msg) => {
        if let ListMsg::Commit(index) = list_msg.clone() {
            model.committed = Some(index);
        }

        update_child(...)
    }
    _ => ...
}
```

This pattern is important because it preserves child reuse while letting the parent express higher-level semantics.

## Derived status and aggregation

The demo machine in `src/demo.rs` is a good example of aggregation.

It composes several child machines and also derives a workspace status string from them.

That shows another common composition role for parents:

- collect child states
- compute a summary or coordination state
- render parent-level status or overview surfaces

## Collaboration-aware composition

Composition should remain compatible with Knopper’s collaboration model.

When composing machines, ask:

1. which child state is purely local?
2. which parent state might later belong in `Shared`?
3. is the parent coordinating local interaction, or semantic shared state, or both?
4. are child IDs stable enough for future overlays?

Examples:

- a command palette’s local focus is participant-local
- a filtered list inside it is local derived state
- a committed command may later lift into shared workspace semantics

## Testing composed machines

Useful tests for composition include:

### Message lifting tests

Verify that child messages become the correct parent messages.

### Child state update tests

Verify that child model state updates through the parent correctly.

### Parent-derived state tests

Verify that the parent computes status, filtered mappings, or selection semantics correctly.

### Projection tests

Verify that the final scene contains the expected composed sections.

### Focus tests

Verify that focus is routed to the intended child or trapped in the intended scope.

The existing command palette and demo tests are good models here.

## Common mistakes to avoid

### 1. Manually rewriting child scenes everywhere

If you are hand-mapping messages repeatedly, `project_child(...)` is probably the better tool.

### 2. Manually rewriting child effects everywhere

If you are hand-translating `Effect<ChildMsg>`, prefer `update_child(...)` or `map_effect(...)`.

### 3. Putting parent semantics into the child unnecessarily

Keep child machines reusable. Put parent-specific meaning in the parent.

### 4. Ignoring focus structure

As soon as a composed machine has multiple interactive children, focus routing becomes part of the design, not an afterthought.

### 5. Using unstable child IDs

Composition depends on stable subtrees and stable anchors, especially for future collaboration overlays.

## Recommended code references

Good places to study next:

- `src/compose.rs`
- `src/standard/list_detail.rs`
- `src/standard/command_palette.rs`
- `src/demo.rs`

## Bottom line

If you remember one sentence from this guide, make it this:

> in Knopper, composing machines means storing child state in the parent, lifting child messages/effects into parent space, and assembling child scenes into a larger semantic scene.
