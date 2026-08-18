## Core thesis

### Knopper should be:

- a functional-reactive TUI framework
- with a terminal-native rendering engine
- backed by Notcurses
- using Cliffy for reactive/geometric state and collaborative state
- using Orlando for compositional transformation/projection pipelines
- using Karpal for lawful algebraic structures, optics, and composition

### And it should not be:
- “React, but in the terminal”
- a virtual DOM clone
- a JSX-first framework
- a widget toolkit with ad hoc mutable internals
  
That distinction is important. Ink’s ergonomics are appealing, but its mental model is still very React-shaped. Knopper can instead be:
 
│ A reactive algebra for terminal scene construction and projection.
 
───────────────────────────────────────────────────────────────────────────────
 
### Recommended architectural shape
 
I’d split Knopper conceptually into 5 layers:
 
### 1. Reactive domain layer
 
This is the app/component state model.
 
Likely built around:
- `Behavior<T>` / `Event<T>` from cliffy-core
- possibly Orlando `Signal<T>` / `Stream<T>` ideas where useful
- optics from Karpal for focused updates
- algebraic state transitions
   
This layer should know nothing about Notcurses.
 
### 2. Component algebra layer
 
This is the novel part. Instead of “components return JSX”, components should produce a terminal scene description.

Something like:
- Text
- Span
- Row
- Column
- Grid
- Box
- Layer
- Viewport
- List
- Input
- Canvas
- Overlay
- Conditional
- Dynamic
- Portal maybe

This should be a typed algebraic DSL, not HTML-shaped. Example direction:

```rust
    let ui = column([
        text("Knopper"),
        row([
            sidebar(files_view),
            panel(editor_view),
        ]),
        status_bar(status_text),
    ]);
```
 
Or more strongly typed:
 
```rust
    let ui = col((                                                                                                                 
       header(title("Knopper")),
       row((
           pane(files()),
           pane(editor()),                                                                                                        
       )),
       footer(status()),
    ));                                                                                                                            
```
 
This is much more terminal-native than fake `<div><span>` structures.

### 3. Projection layer

This is where your “stand-alone Cliffy pattern” matters.

A component should not directly mutate terminal planes. It should project into a scene graph or render graph.

I’d recommend:
- App model
- -> reactive component graph
- -> projected terminal scene
- -> layouted plane graph
- -> Notcurses commands

So instead of React’s VDOM diff, Knopper does:

│ FRP state -> scene projection -> structural diff/patch -> terminal render commit

This is a better fit for TUI than “re-render everything”.

### 4. Rendering layer

This is Notcurses-specific.

Responsibilities:
- plane allocation/reuse
- cell styling
- unicode/grapheme handling
- z-order/layers
- image/block/sextant/braille rendering where relevant
- minimal diffing between previous and next scene
- final atomic render commit

This layer should be isolated behind a renderer trait.

### 5. Runtime/event layer

This handles:
- keyboard/mouse/paste/resize input
- timers
- subscriptions
- async task completion
- external sync events from cliffy-protocols

It should feed an event stream into the reactive graph.

────────────────────────────────────────────────────────────────────────────────

### The most important design choice

#### Do not make the component primitive “render function” - Make it a reactive machine.

Something like:
- input events in
- local/global state evolves
- scene projection out

Conceptually:
```text                                                                                                                          
  Component = Context × EventStream<Input> -> Behavior<Scene>
```                                                                                                                              
                                                                                                                                  
or                                                                                                                               
                                                                                                                                  
```text
  Component = Machine<Model, Msg, Scene>
```

Where:
- Model is local state
- Msg is an algebraic event/intention type
- Scene is a terminal scene tree

That gives you something closer to:
- FRP
- transducers
- algebraic state machines

than React’s:
- props
- hooks
- reconciliation

────────────────────────────────────────────────────────────────────────────────

### A possible Knopper component model

I think a good core abstraction is:

```rust
  Widget<Ctx, Msg>                                                                                                                 
```

A widget is a pure/referentially transparent projector over reactive inputs.

It can:
- read context
- emit messages
- describe scene
- compose with other widgets

Then separately, a runtime-managed wrapper can add:
- focus
- subscriptions
- async commands
- lifecycle

You may also want a more explicit machine form:

```rust
  Component<Model, Msg>
```
  
with:
- init() -> Model
- update(msg, model) -> Model
- view(model) -> `Scene<Msg>`
- subscriptions(model) -> `Stream<Msg>`

That looks Elm-ish, but the internal execution can still be Cliffy/Orlando-based rather than TEA-style message loops.

I actually think the sweet spot is a hybrid:
- FRP for state propagation
- message algebra for intent boundaries
- pure scene projection for rendering

So:
- use Behavior / Event internally
- expose clean typed component composition externally

────────────────────────────────────────────────────────────────────────────────

### Where each IA library fits

### Cliffy

Best use:
- reactive state
- time-varying values
- derived behaviors
- event streams
- geometric state for layout/focus/spatial composition
- CRDT/sync for collaborative TUIs

### Specific Cliffy opportunities

#### cliffy-core

Use for:
- `Behavior<T>`
- `Event<T>`
- combinators
- maybe geometric state for:
  - cursor/focus trajectories
  - panel geometry
  - animated transitions
  - layout interpolation
  
#### cliffy-protocols

Use optionally for:
- collaborative TUI state
- multi-user dashboards
- shared terminal workspaces
- synchronized operator consoles
- distributed command/control surfaces

This could become a killer differentiator:

│ Knopper apps can be collaborative by construction.

Not just “networked terminal apps”, but algebraically synchronized scene/application state.

────────────────────────────────────────────────────────────────────────────────

### Orlando

This seems ideal for the projection pipeline.

Use Orlando’s transducer ideas for:
- input normalization pipelines
- event routing
- scene transformation
- layout passes
- patch generation
- render command fusion

Example conceptual pipeline:

```text
  RawInput
    -> normalize
    -> route_to_focus
    -> lift_to_msg
    -> update_model
    -> project_scene
    -> layout
    -> diff
    -> render_ops                                                                                                                  
 ```
 
 That is extremely Orlando-shaped.
 
 I would especially avoid using Orlando merely as a utility crate. It feels central to Knopper’s architecture:
 - components compose as transformations
 - projection is a transducer pipeline
 - terminal patches are reducible streams
 
 ────────────────────────────────────────────────────────────────────────────────
 
 ### Karpal
 
 Karpal should provide the laws and abstractions that keep Knopper from becoming a bag of framework hacks.
 
 Best uses:
 - optics for nested app/component state
 - semigroups/monoids for style merging and patch accumulation
 - prisms for event unions / message routing
 - profunctor optics for scene focus/transforms
 - arrows/categories for component composition if useful
 - recursion schemes if you want principled tree folds over scene graphs
 
 #### High-value Karpal applications
 
 #### 1. Style algebra
 
 Styles should be lawful compositional values.
 
 ```rust                                                                                                                          
   Style + Style -> Style                                                                                                         
 ```
 Use semigroup/monoid semantics for:
 - fg/bg color composition
 - attributes
 - border themes
 - inheritance/defaulting
 
 #### 2. Optics
 
 Very useful for nested state updates:
 - app model -> pane model
 - pane model -> widget model
 - widget model -> focused field
 
 This lets parent components host child state without ugly mutation plumbing.
 
 #### 3. Sum-type routing
 
 Prisms for:
 - app-level message enums
 - variant-focused handlers
 - event routing into subcomponents
 
 ────────────────────────────────────────────────────────────────────────────────
 
 ### Notcurses-specific architectural advice
 
 Notcurses is powerful, but it changes the design constraints.
 
 #### Important principle:
 
 Render prep may be parallel; Notcurses commit should likely remain single-threaded.
 
 So:
  - use rayon for:
    - layout calculations
    - diff preparation
    - text measurement
    - off-thread patch computation
  - but keep final plane mutation / render commit serialized unless Notcurses guarantees more
  
#### Knopper should model Notcurses as:

  - a terminal scene backend
  - not as the primary app model
  
That means don’t let components manipulate planes directly.

Instead:
  - components produce scene nodes
  - renderer lowers nodes into planes/cells
  
This will preserve portability and testability.

────────────────────────────────────────────────────────────────────────────────

### Suggested scene model

I’d recommend three intermediate representations.

1. Declarative scene tree
  
Human-friendly component output.
  
Example variants:
  - Text
  - Row
  - Col
  - Grid
  - Box
  - Scroll
  - Overlay
  - Spacer
  - Canvas
  - Input
  - List
  - Conditional
  - Dynamic
  
2. Layouted scene
  
After measurement/layout:
  - absolute coordinates
  - sizes
  - clipping
  - z-index
  - style resolution

3. Render ops                  

 Backend-specific:
  - write grapheme
  - set style
  - move cursor
  - create plane
  - resize plane
  - blit image
  - draw border
  
This separation will make Knopper much easier to test.

────────────────────────────────────────────────────────────────────────────────

### I strongly agree with avoiding HTML-like syntax.

Good direction

A DSL that feels like:
  - terminal composition
  - geometry
  - algebraic layout
  - reactive values
  
Example:

```rust
    let app = dock(
        top(status_bar(status)),
        left(width(24, file_tree(files))),
        center(editor(buffer)),
        bottom(height(1, command_line(input))),
    );                                                                                                                             
 ```
 
Or:
 
```rust
    let app = col((
        text("Knopper"),
        when(show_sidebar, row((
            width(24, sidebar()),
            fill(main_panel()),
        ))),
        footer(status_line()),
    ));
```

This is much closer to TUI reality than pseudo-DOM.

Start with pure Rust builders

I would not begin with proc macros.

Start with:
- enums
- builders
- combinators
- functions

Then later, if desired, add syntax sugar.

That will keep the semantics honest.

────────────────────────────────────────────────────────────────────────────────

A component architecture I think fits your vision

Here’s the shape I’d currently recommend.

Components as algebraic machines

```rust
    trait Component {
        type Model;
        type Msg;
        type Output;
        
        fn init(&self) -> Self::Model;
        fn update(&self, model: &mut Self::Model, msg: Self::Msg);
        fn project(&self, model: &Self::Model) -> Self::Output;
    }
```

But internally, project can lift into reactive graph semantics.

A more FRP-native form would be:

```rust
    trait ReactiveComponent {
        type Input;
        type Msg;
        type Scene;
        
        fn bind(&self, input: Event<Self::Input>) -> Behavior<Self::Scene>;
    }
```

I think the best practical choice is:
- public API closer to `Component<Model, Msg>`
- implementation/runtime internally powered by Cliffy behaviors/events

That gives users clarity without exposing every FRP detail.

────────────────────────────────────────────────────────────────────────────────

### A novel Knopper-specific idea

“Projection Machines” instead of components

If you want to differentiate from React/Ink more strongly, call the primitive something other than component.

Maybe:
- Projection
- Machine
- Pane
- Node
- Surface
- Knop if you want project branding

For example:

```rust
    trait Projection {
        type State;
        type Event;
        fn scene(&self, state: &Self::State) -> Scene<Self::Event>;
    }                                                                                                                              
```
 
That nomenclature alone helps avoid React cargo-culting.

────────────────────────────────────────────────────────────────────────────────

### Collaboration/sync story

This is where Knopper could be genuinely novel.

Most TUI frameworks stop at local interactivity.

Knopper could support:
- multi-user shared state
- replicated operator dashboards
- synchronized terminal workflows
- remote cursors/focus
- collaborative editors/forms/panels

Using cliffy-protocols, you could treat:
- local UI state
- remote updates
- derived scene state

as one convergent reactive system.

That is much more interesting than “Ink in Rust”.

────────────────────────────────────────────────────────────────────────────────

### Risks / pitfalls

#### 1. Over-importing web UI assumptions

Avoid:
  - React hooks semantics
  - VDOM-first thinking
  - DOM-like node names
  - browser-style event bubbling unless carefully adapted

Terminals are different:
  - focus is more explicit
  - layout is tighter
  - rendering cost model is different
  - text and glyph width are hard constraints
  
#### 2. Overusing geometric abstraction

Cliffy geometry should serve:
  - layout
  - motion
  - interpolation
  - merge/sync
  
It should not leak everywhere if it makes common TUI work awkward.

#### 3. Letting Notcurses leak into public APIs

Public API should talk in:
  - scene
  - style
  - layout
  - surface
  - event
  
not in raw Notcurses planes/cells.

#### 4. Premature macro DSL work

The algebra should stabilize before syntax sugar.

────────────────────────────────────────────────────────────────────────────────

### What I would build first

#### Milestone 1: a tiny lawful core

Before fancy widgets, prove the architecture with:

##### Core crates/modules
  - knopper-core
    - scene algebra
    - style algebra
    - layout primitives
    - component/projection traits
  - knopper-runtime
    - event loop
    - subscriptions
    - FRP wiring
  - knopper-notcurses
    - renderer backend
  - knopper-widgets
    - text, row, col, box, input, list
    
##### Demo capabilities
  - reactive counter
  - focusable list
  - text input
  - split panes
  - resize handling
  - diffed redraws
  
That would validate the model fast.

────────────────────────────────────────────────────────────────────────────────

#### My recommendation in one sentence

Build Knopper as:

```text
a typed reactive scene algebra for terminal interfaces, projected through Orlando-style transformation pipelines into a Notcurses renderer, with Cliffy-powered state/sync and Karpal-backed lawful composition.
```

That feels coherent and genuinely IA-native.

────────────────────────────────────────────────────────────────────────────────

Concrete next design questions

I think the next useful discussion is to decide these 4 things:

1. What is the primary public abstraction?
  - Component
  - Projection
  - Widget
  - Machine
2. What does a component return?
  - `Scene<Msg>`
  - `Behavior<Scene<Msg>>`
  - something hybrid
3. Is local state message-driven, FRP-driven, or hybrid?
  - TEA-like
  - pure FRP
  - hybrid
4. What is the minimal scene algebra?
  - text/layout only
  - or include input/focus/scroll in v0
