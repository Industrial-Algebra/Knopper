# Writing a Machine in Knopper

## Purpose

This guide explains how to author a custom `Machine` in Knopper.

It is the most important user-facing programming guide in the framework, because `Machine` is Knopper’s primary public abstraction.

If you understand how to write a machine, you understand the core shape of Knopper.

## The mental model

A Knopper machine is not a component in the DOM sense.

A machine is a semantic projection with four key parts:

- **`Context`**: static or externally supplied configuration
- **`Model`**: local machine state
- **`Shared`**: shared or externally synchronized semantic state
- **`Msg`**: events/messages that update the machine

A machine does three things:

1. initializes its local model with `init(...)`
2. updates its model in response to messages with `update(...)`
3. projects semantic UI as `Behavior<Scene<Msg>>`

In other words:

> a machine is a stateful semantic process that projects a scene.

## The `Machine` trait

The core trait is:

```rust
pub trait Machine {
    type Context;
    type Msg;
    type Model;
    type Shared;

    fn init(&self, ctx: &Self::Context) -> Self::Model;

    fn update(
        &self,
        model: &mut Self::Model,
        msg: Self::Msg,
        ctx: &Self::Context,
    ) -> Effect<Self::Msg>;

    fn project(
        &self,
        model: Behavior<Self::Model>,
        shared: Behavior<Self::Shared>,
        ctx: &Self::Context,
    ) -> SceneBehavior<Self::Msg>;

    fn project_once(
        &self,
        model: &Self::Model,
        shared: &Self::Shared,
        ctx: &Self::Context,
    ) -> Scene<Self::Msg>;

    fn cursor_position(
        &self,
        model: &Self::Model,
        shared: &Self::Shared,
        ctx: &Self::Context,
        layout: &LayoutNode,
    ) -> Option<(u16, u16)>;
}
```

You usually implement at least:

- `init`
- `update`
- `project_once` or `project`

And optionally:

- `cursor_position`

## State lanes: `Model` vs `Shared`

Knopper is collaboration-aware, so you should think carefully about state placement.

### Put state in `Model` when it is local

Examples:

- local cursor position
- local focus-related mechanics
- local scroll offset
- local draft editing state
- temporary UI transitions

### Put state in `Shared` when it is semantic and externally synchronizable

Examples:

- shared document contents
- shared task state
- replicated annotations
- participant registry
- synchronized command history

### Important rule

Do not put shared semantic truth into `Model` just because a first draft is easier.

For `0.1.0`, some standard machines are intentionally local-first, but custom machine authors should still think in terms of these two state lanes.

## Messages and effects

Messages are the machine’s update language.

A message should represent a semantic intent, not a renderer detail.

Examples:

- `Increment`
- `Toggle`
- `Commit`
- `Select(usize)`
- `Insert(char)`

The `update(...)` function returns an `Effect<Self::Msg>`.

Current effect forms include:

- `Effect::None`
- `Effect::Emit(msg)`
- `Effect::Batch(vec![...])`
- `Effect::RequestFocus(node_id)`

This allows a machine to:

- mutate local model state
- trigger follow-up messages
- request focus movement

## Your first machine

A simple machine often starts with four types:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterContext {
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CounterState {
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CounterMsg {
    Increment,
}

#[derive(Debug, Clone, Default)]
pub struct CounterMachine;
```

Then implement `Machine`.

## Example: local counter machine

```rust
use knopper::{Effect, Machine, Role, Scene, SceneBehavior};
use cliffy_core::{Behavior, behavior};

impl Machine for CounterMachine {
    type Context = CounterContext;
    type Msg = CounterMsg;
    type Model = CounterState;
    type Shared = ();

    fn init(&self, _ctx: &Self::Context) -> Self::Model {
        CounterState::default()
    }

    fn update(
        &self,
        model: &mut Self::Model,
        msg: Self::Msg,
        _ctx: &Self::Context,
    ) -> Effect<Self::Msg> {
        match msg {
            CounterMsg::Increment => {
                model.value += 1;
                Effect::None
            }
        }
    }

    fn project_once(
        &self,
        model: &Self::Model,
        _shared: &Self::Shared,
        ctx: &Self::Context,
    ) -> Scene<Self::Msg> {
        Scene::text(1_u64, format!("{}: {}", ctx.label, model.value))
            .with_role(Role::StatusLine)
            .on_activate(CounterMsg::Increment)
    }

    fn project(
        &self,
        model: Behavior<Self::Model>,
        _shared: Behavior<Self::Shared>,
        ctx: &Self::Context,
    ) -> SceneBehavior<Self::Msg> {
        let scene = behavior(self.project_once(&model.sample(), &(), ctx));
        let scene_for_model = scene.clone();
        let ctx = ctx.clone();
        model.subscribe(move |next_model| {
            scene_for_model.set(CounterMachine.project_once(next_model, &(), &ctx));
        });
        scene
    }
}
```

This is the core shape of many local-first machines in Knopper today.

## `project_once(...)` vs `project(...)`

Knopper provides both a one-shot projection shape and a reactive projection shape.

### `project_once(...)`

Use this to define how the machine turns a single `(model, shared, ctx)` snapshot into a `Scene`.

This is usually the clearest place to express the machine’s view logic.

### `project(...)`

Use this when you want a full `Behavior<Scene<Msg>>`.

In many current machines, `project(...)` is implemented by:

1. computing an initial scene with `project_once(...)`
2. subscribing to `model`
3. re-projecting whenever the model changes

That pattern is a good default.

## Choosing IDs

Every meaningful interactive or semantically stable surface should have a stable `NodeId`.

Good IDs are:

- stable across reprojection
- tied to semantic identity
- predictable enough for testing and future overlays

Avoid IDs that change arbitrarily from frame to frame.

This matters for:

- focus
- activation
- diffing
- backend patch stability
- future collaboration overlays

## Scenes are semantic UI, not DOM widgets

Your machine projects a `Scene<Msg>`.

Common constructors include:

- `Scene::text(...)`
- `Scene::row(...)`
- `Scene::column(...)`
- `Scene::border(...)`
- `Scene::padding(...)`
- `Scene::sized(...)`
- `Scene::stack(...)`
- `Scene::align(...)`
- `Scene::focus_scope(...)`

Scenes carry:

- structure
- styling
- annotations
- focusability
- activation behavior

Example:

```rust
Scene::border(
    root_id,
    Scene::sized(
        inner_id,
        SizeConstraint::width(24),
        Scene::text(label_id, "Run")
            .focusable()
            .on_activate(MyMsg::Run),
    ),
)
```

## Focus and interaction

If a node should participate in keyboard focus traversal, mark it focusable:

```rust
Scene::text(id, "item").focusable()
```

If a node should emit a message on activation:

```rust
Scene::text(id, "run").on_activate(MyMsg::Run)
```

Usually `on_activate(...)` also implies focusability for text nodes.

If your machine has a local traversal region, consider using a focus scope:

```rust
Scene::focus_scope_with_policy(
    scope_id,
    "my-scope",
    knopper::FocusScopePolicy::Passthrough,
    body,
)
```

## Cursor placement

If your machine owns a text cursor, implement `cursor_position(...)`.

This method is called after layout has been resolved, so it can compute cursor placement from final geometry.

This is how `InputMachine` and `TextareaMachine` report cursor position.

Typical pattern:

1. find the relevant layout node with `find_node(...)`
2. compute cursor coordinates from layout rect plus local cursor state
3. return `(x, y)`

This is better than burying cursor logic in rendering code.

## Using `PureMachine`

For many simple cases, `PureMachine` is a convenient shortcut.

It lets you construct a machine from closures:

```rust
let machine = PureMachine::new(
    |_ctx: &Ctx| 0_i32,
    |model: &mut i32, msg: Msg, _ctx: &Ctx| {
        match msg {
            Msg::Increment => *model += 1,
        }
        Effect::None
    },
    |model: &i32, _shared: &(), _ctx: &Ctx| {
        Scene::text(1_u64, model.to_string())
    },
);
```

Use `PureMachine` when:

- the machine is small
- closure-based construction stays readable
- you do not need a more structured concrete type yet

Use a dedicated struct when:

- the machine has helper methods
- it needs stable IDs or configuration helpers
- it composes child machines
- the logic is becoming substantial

## Composing machines

Knopper supports nested machines.

The most important helpers are:

- `project_child(...)`
- `update_child(...)`
- `map_effect(...)`
- `dispatch_if_focused(...)`
- `child_has_focus(...)`

### Projecting a child machine

```rust
let child_scene = project_child(
    &self.child,
    &model.child,
    &shared.child,
    &ctx.child,
    &ParentMsg::Child,
);
```

This projects the child scene and lifts child messages into parent messages.

### Updating a child machine

```rust
let effect = update_child(
    &self.child,
    &mut model.child,
    child_msg,
    &ctx.child,
    &ParentMsg::Child,
);
```

This applies a child update and lifts any returned effects into the parent message space.

### Focus-aware dispatch

If a parent machine routes keys to one child at a time, helpers such as `dispatch_if_focused(...)` and `child_has_focus(...)` are useful.

This pattern appears in composed machines like the command palette.

## Example: parent composition shape

At a high level, a composed machine often looks like this:

```rust
match msg {
    ParentMsg::Child(child_msg) => update_child(
        &self.child,
        &mut model.child,
        child_msg,
        &ctx.child,
        &ParentMsg::Child,
    ),
    ParentMsg::Other => { /* parent-local logic */ }
}
```

And in projection:

```rust
let child = project_child(
    &self.child,
    &model.child,
    &shared.child,
    &ctx.child,
    &ParentMsg::Child,
);

Scene::column(root_id, vec![child, status])
```

## Collaboration-aware machine authoring

Even if your first machine is local-first, ask these questions.

1. Which state is purely local?
2. Which state might become shared later?
3. Which state might become participant-local published state?
4. Are my `NodeId`s stable enough for overlays?
5. Am I accidentally assuming one globally authoritative focus or cursor?

Examples:

- a local text cursor usually belongs in `Model`
- shared document text likely belongs in `Shared`
- remote cursor presence would later be derived overlay state

## Testing machines

Knopper strongly benefits from TDD.

Useful tests include:

### Update tests

Verify that messages update model state correctly.

### Projection tests

Verify that the machine projects the right scene or render ops.

### Cursor tests

For text-oriented controls, verify cursor position from layout.

### Composition tests

When composing children, verify that:

- child messages are lifted correctly
- focus dispatch is scoped correctly
- parent status/derived state stays coherent

## A practical checklist

When writing a machine, it is usually worth checking the following.

### Define the machine surface

- `Context`
- `Model`
- `Shared`
- `Msg`
- machine struct

### Implement behavior

- `init(...)`
- `update(...)`
- `project_once(...)`
- `project(...)`
- optional `cursor_position(...)`

### Choose stable IDs

- root
- interactive nodes
- semantic sub-surfaces

### Add tests

- update behavior
- rendering/projection
- focus or activation behavior
- cursor behavior if relevant

## Study the existing machines

Good examples in the current tree include:

- `src/standard/input.rs`
  - local-first text editing
  - machine-derived cursor
- `src/standard/textarea.rs`
  - multiline local editing
  - line-stable IDs
- `src/standard/list.rs`
  - semantic selection vs commit
- `src/standard/command_palette.rs`
  - child-machine composition
  - modal/focus behavior
- `src/demo.rs`
  - composed workspace built from multiple machines

## Common mistakes to avoid

### 1. Putting too much in `Context`

`Context` should be configuration or externally supplied input, not hidden mutable state.

### 2. Using unstable IDs

If IDs shift unpredictably, focus, activation, diffing, and future collaboration overlays become harder.

### 3. Mixing semantic state with renderer state

Keep semantic state in `Model`/`Shared`, not in rendering/backend details.

### 4. Treating `Model` as the final home for everything

A local-first prototype is fine, but be explicit when a later shared split is likely.

### 5. Encoding focus behavior ad hoc in every control

Prefer structural focus semantics and shared helpers where possible.

## What to read next

After this guide, useful next references are:

- `docs/architecture/01-machine-model.md`
- `docs/architecture/02-scene-algebra.md`
- `docs/architecture/04-runtime-pipeline.md`
- `docs/roadmap/01-collaboration-readiness.md`
- `src/standard/input.rs`
- `src/standard/command_palette.rs`
- `src/demo.rs`

## Bottom line

If you remember only one sentence, remember this:

> a Knopper machine owns semantic local/update logic and projects a scene from local model state, shared state, and context.

That is the center of the framework.
