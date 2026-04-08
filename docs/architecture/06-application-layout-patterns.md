# Application Layout Patterns

## Purpose

This note proposes a new layer in Knopper above raw `Scene` composition and below app-specific `Machine` code:

- an **application layout pattern layer**
- a **semantic panel/split/region system** for terminal apps
- a future foundation for richer **Notcurses-native effects and animation**

The immediate motivation is practical: Knopper now has two demos with enough complexity to expose recurring layout problems that are awkward to solve with ad hoc `Border + Padding + Sized + Row + Column` assembly.

The broader motivation is architectural: if Knopper is going to become a strong Machine-centered framework for collaboration-ready terminal applications, it needs a reusable language for application structure, not just individual controls.

## Status

This is a design note, not a committed public API.

It describes:

- the problem being observed
- why current primitives are not quite enough on their own
- what an additional layout layer should provide
- how that layer should relate to Knopper’s scene algebra and Notcurses ambitions

## Problem statement

Knopper’s existing scene algebra already provides useful structural primitives:

- `Row`
- `Column`
- `Stack`
- `Align`
- `Padding`
- `Border`
- `Sized`
- `Viewport`
- `Scroll`
- `FocusScope`

These are good foundational operators.

But current demos repeatedly run into higher-level layout problems such as:

- border and padding chrome not being accounted for consistently across nested panes
- sibling panes negotiating width/height poorly under tight terminal bounds
- detail panels growing beyond their intended region
- list/detail splits and editor/detail splits requiring repeated one-off sizing math
- ad hoc truncation/wrapping logic for bounded content regions
- app surfaces needing repeated header/subtitle/presence/body composition
- layout assumptions leaking into demo-specific helper code instead of becoming reusable patterns

In other words:

> Knopper has strong local scene primitives, but it lacks a reusable language for application-scale layout.

## Architectural direction

The proposed direction is:

> add a semantic application layout layer above raw `Scene` composition and below full app `Machine`s.

This layer should not replace the existing scene algebra.

Instead, it should:

- be implemented in terms of the scene algebra
- provide stable, reusable layout contracts for app surfaces
- reduce repeated manual chrome accounting
- make bounded regions and split layouts explicit
- create a clean place to later expose Notcurses-native presentation/effect concepts

## Non-goals

This proposal is **not** an attempt to add:

- CSS
- a DOM clone
- a stringly styling language
- a browser-like layout engine
- hidden imperative widget geometry

Knopper should remain:

- terminal-native
- scene-structural
- Machine-centered
- semantically explicit

The goal is closer to:

- algebraic app layout combinators
- track/region/surface composition
- explicit bounded regions and overflow rules

than to anything resembling web styling.

## Why raw `Scene` is not enough on its own

Raw `Scene` composition is still the right substrate.

But once application authors repeatedly write structures like:

- bordered titled panel
- panel with subtitle and presence strip
- list/detail split
- queue/editor split
- bounded detail pane with truncation policy
- header/body/footer shell

then the problem is no longer “how to build a scene”, but:

- how to express recurring **layout intent**
- how to keep pane contracts stable
- how to avoid re-solving geometry/chrome interactions each time

So the missing layer is not lower than `Scene`, but one level higher:

- a reusable app layout pattern layer

## Proposed layering

The intended stack becomes:

1. **Raw scene algebra**
   - `Scene`, `Padding`, `Border`, `Sized`, `Stack`, etc.
2. **Application layout patterns**
   - panels, splits, shells, bounded regions, detail surfaces
3. **Reusable standard machines**
   - list, input, textarea, toggle, command palette, etc.
4. **Application machines**
   - composed workspaces, editors, dashboards, review flows, collaborative tools

This is important because not every reusable application structure should become a stateful machine.

Many application-level patterns are:

- structural
- presentational
- bounded-layout concerns
- not independently stateful

Those belong in layer 2, not layer 3.

## Candidate primitives

The following are candidate concepts for the new layer.

These are not final names.

### 1. Surface primitives

A **surface** is an application-scale framed region with predictable internal structure.

A useful refinement is to separate:

- `panel_header(...)`
- `panel_chrome(...)`
- bounded body content

so that header lanes, background fills, and body-region policies can evolve somewhat independently.

Examples:

- titled panel
- panel with subtitle
- panel with header metadata
- panel with presence strip
- detail panel

Possible helpers:

- `panel(...)`
- `panel_with_subtitle(...)`
- `panel_with_presence(...)`
- `detail_panel(...)`
- `section(...)`

Responsibilities:

- own border/padding/header chrome accounting
- define the body slot size contract
- expose body/content as a bounded region
- optionally reserve header/footer lanes
- create a clean future seam for Notcurses-native panel fills/effects

This avoids each app manually stacking `Border + Padding + Sized + Column` in slightly different and fragile ways.

### 2. Split primitives

Application UIs often need stable splits.

Examples:

- sidebar + main pane
- list + detail
- queue + editor
- editor + inspector

Possible helpers:

- `split_h(...)`
- `split_v(...)`
- `split_h_fixed_left(...)`
- `split_h_fixed_right(...)`
- `split_h_ratio(...)`
- `master_detail(...)`

Responsibilities:

- define slot allocation rules explicitly
- bound children to negotiated slots
- keep chrome accounting separate from content sizing
- make responsive behavior less ad hoc

A list/detail split should be a reusable pattern, not something re-derived from raw `Row` math in each demo.

### 3. Shell primitives

Many apps want a top-level shell.

Examples:

- header / body / footer
- toolbar / workspace / status
- title / controls / main / footer

Possible helpers:

- `app_shell(...)`
- `header_body_footer(...)`
- `workspace_shell(...)`

Responsibilities:

- define a stable top-level arrangement
- keep body height bounded
- reserve footer/status space structurally
- make outer-app layout readable and reusable

### 4. Bounded region / overflow policies

A key recurring problem is overflow.

Knopper needs clearer reusable policies for bounded content areas.

Possible policies:

- `Clip`
- `Wrap`
- `Truncate`
- `Scroll`
- `Auto` (carefully, if at all)

These should apply to:

- detail panes
- summary text
- panel body slots
- future timeline/log/list views

The important thing is to make the policy structural and explicit, not hidden inside ad hoc text formatting.

### 5. Region naming and semantic slots

For more complex applications, named regions may become useful.

Examples:

- `header`
- `body`
- `sidebar`
- `detail`
- `footer`
- `status`
- `overlay`

These names could matter for:

- focus behavior
- effects
- animation
- presence overlays
- future responsive substitution rules

This does **not** necessarily imply a string-based CSS grid template.

It could instead be modeled with structured enums or typed region descriptors.

## Relationship to existing `Scene` / `LayoutKind`

The application layout layer should ideally be implemented as one of:

1. **pure helper functions returning existing `Scene` values**, or
2. **new scene wrappers / layout kinds** only when the lower layer genuinely needs more semantic information

The preferred first step is:

- helper functions / small structs built on the current scene algebra

Only after that should Knopper consider adding new core scene/layout variants.

That keeps experimentation cheaper and reduces the risk of prematurely freezing the wrong abstraction.

## Relationship to Machines

This layer should complement Machines, not compete with them.

### Good candidates for application layout helpers

- bordered shells
- titled surfaces
- split layout contracts
- bounded detail panes
- presence strips
- metadata rows

### Good candidates for Machines

- text editors
- selectable lists
- toggles
- tabs
- command palette
- task board with local interaction state
- collaborative editor with semantic message flow

Rule of thumb:

> if the thing needs local state, input semantics, or its own update language, it probably wants to be a Machine.
>
> if the thing mostly provides reusable structure and bounded layout, it probably belongs in the application layout layer.

## Relationship to collaboration

This layer is especially important for collaboration-readiness.

Why?

Because collaborative applications often need participant-local app surfaces such as:

- presence sidebars
- collaborator badges in headers
- participant-local focus rings
- local overlays in a detail region
- review/annotation lanes

If application layout remains ad hoc, these become fragile.

If layout regions become explicit, Knopper gains a clean place to attach:

- participant-local overlays
- presence adornments
- shared-vs-local region distinctions
- collaboration metadata at the app shell level

So this layer is not merely visual polish.

It is part of Knopper’s collaboration architecture.

## Relationship to Notcurses effects and animation

This is one of the strongest reasons to pursue the layer carefully.

Knopper’s future may want to expose richer Notcurses-native affordances such as:

- panel transitions
- region-local fades or reveals
- z-aware overlay choreography
- animated emphasis for focus/selection/presence
- motion between semantic regions
- bounded-region redraw strategies optimized for Notcurses

A semantic application layout layer provides better anchors for that than raw unstructured scene composition.

For example, a split between `panel_header(...)` and `panel_chrome(...)` creates a natural place to later support:

- animated header emphasis
- panel background fills (including grid-like treatments)
- focus-reactive panel fill promotion
- bounded body reveal effects
- region-local redraw and effect policies

For example, if Knopper knows something is a:

- detail region
- sidebar
- header surface
- modal surface
- bounded panel body

then future effect APIs can target those regions semantically.

That is much better than asking users to manually animate arbitrary low-level node trees.

So this proposal may become the foundation for exposing richer Notcurses capabilities **without collapsing the framework into backend-specific hacks**.

## Possible first implementation strategy

A practical incremental plan:

### Phase 1: helper-layer only

Implement app-layout helpers outside the core scene algebra.

Candidates:

- `panel(...)`
- `panel_with_presence(...)`
- `split_h_fixed(...)`
- `header_body_footer(...)`
- `detail_block(...)`
- overflow helpers for text/detail slots

Use these to refactor both demos.

### Phase 2: compare both demos

After both demos are using the helper layer, identify:

- what stayed stable
- what was demo-specific
- what still leaked chrome/layout math
- what needs richer structural representation

### Phase 3: promote stable abstractions

Only then consider:

- moving some helpers into a more formal module
- introducing stronger typed layout descriptors
- possibly extending `Scene` / `LayoutKind` if needed

This keeps the architecture evidence-driven.

## Immediate next questions

Before implementation, the following questions should be answered:

1. What is the minimum viable panel/split API?
2. How should bounded overflow be expressed structurally?
3. Should region naming be string-based, enum-based, or typed?
4. Which demo helper patterns are stable enough to generalize now?
5. Which concerns belong in helper functions versus core scene/layout kinds?
6. How can future Notcurses effects target semantic regions without hardcoding backend details?

## Recommendation

Proceed with a first pass implemented as:

- reusable helper-layer patterns
- built on existing `Scene` primitives
- exercised by both demos

Do **not** jump directly to a full grid engine.

Instead, build toward:

- surfaces
- shells
- splits
- bounded regions
- overflow policies

If those stabilize across both demos, they can later inform a richer application layout algebra.

## Summary

Knopper now needs more than local scene primitives and individual controls.

It needs a reusable language for application structure.

The right direction is:

- not CSS
- not a DOM clone
- not hidden layout magic

but rather:

- a semantic application layout layer
- structurally expressed in terms of scenes
- complementary to Machines
- suitable for collaboration-aware applications
- and likely foundational for future Notcurses-native effects and animation
