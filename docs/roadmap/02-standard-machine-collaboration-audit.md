# Knopper standard-machine collaboration audit

## Purpose

This document audits the current standard-machine set against the collaboration-readiness guide.

The goal is not to require full collaborative behavior immediately. The goal is to identify where the current machines are already compatible with Knopper’s collaborative direction, where they are merely local-first simplifications, and where they risk hardening single-user assumptions into public APIs.

## Scope

Current audited modules:

- `src/standard/input.rs`
- `src/standard/list.rs`
- `src/standard/list_detail.rs`
- `src/standard/modal.rs`
- `src/standard/command_palette.rs`

## Audit rubric

For each machine/helper, assess:

1. which state is local ephemeral
2. which state may become shared
3. which state may become participant-local published state
4. whether the API assumes a single global focus or selection owner
5. whether stable scene identity is preserved well enough for collaboration overlays
6. what follow-up work is needed before or after `0.1.0`

## Summary

Overall assessment: **good collaboration direction, but still strongly local-first in standard-machine state modeling**.

That is acceptable for `0.1.0` as long as the project remains explicit about the distinction between:

- local-first controls for current use
- and collaboration-capable architectural seams for future shared and participant-local state

The strongest positive findings are:

- `Machine` already separates `Model` and `Shared`
- focus currently lives in runtime-local state rather than pretending to be globally shared
- scene/node identity is explicit and stable enough for many overlay use cases
- modal and focus behavior are increasingly structural rather than widget-hardcoded

The main risks are:

- several standard machines use `Shared = ()`, which is fine for now but easy to over-interpret as the final form
- selection and commit semantics are currently modeled as purely local machine state even where downstream applications may want shared semantics
- input/editing controls are explicitly local-first and should not be mistaken for collaborative editing abstractions

## Machine-by-machine audit

## 1. `InputMachine`

### Current public shape

- `Model = InputState`
- `Shared = ()`
- state:
  - `value`
  - `cursor`
  - `committed`

### Collaboration assessment

#### Local ephemeral state

Clear local state today:

- `cursor`
- transient editing mechanics
- likely any future IME/composition details

#### Potentially shared state

State that may be shared in some downstream uses:

- `value`
- `committed`

In some applications, the current text field content is purely local draft state. In others, it may represent shared semantic state such as a synchronized filter query, collaborative command surface, or replicated form field.

#### Participant-local published state

Future publishable state may include:

- cursor location
- selection range if input gains selection semantics
- active editor ownership/presence

### Risk level

**Medium**

The current abstraction is fine as a local input field, but it should not be treated as the long-term collaborative text editing abstraction.

### Good signs

- stable node IDs via `root_id` and `field_id`
- cursor is machine-derived from layout rather than being buried in renderer logic
- local-first shape is simple and predictable

### Risks

- `value` living entirely in local `Model` may encourage downstream code to couple semantic text state to local editing state
- no distinction yet between shared content and participant-local editing/presence state

### Recommendation

For `0.1.0`, document `InputMachine` as a **local-first field control**.

Do not present it as the future collaborative editor model.

### Follow-up

- add explicit docs noting that collaborative editors will likely separate shared content from participant-local cursor/composition state
- when editor work begins, treat that as a separate abstraction rather than overextending `InputMachine`

## 2. `ListMachine`

### Current public shape

- `Model = ListState`
- `Shared = ()`
- state:
  - `selected`
  - `scroll`

### Collaboration assessment

#### Local ephemeral state

- `scroll` is typically participant-local
- focus within list is participant-local

#### Potentially shared state

- `selected` may be shared in some collaborative applications
- committed choice may map to shared application state upstream

#### Participant-local published state

Potential future publishable state:

- local current row/selection highlight as presence
- viewport anchor or remote attention marker

### Risk level

**Low to medium**

The current API is reasonably adaptable, but its docs should avoid implying that list selection is always local or always shared.

### Good signs

- scene IDs are stable and structured (`root`, per-item IDs, per-marker IDs)
- list machine already emits semantic messages like `Select` and `Commit`
- parent machines can decide what selection/commit means upstream

### Risks

- `selected` currently lives only in local `Model`
- downstream users may assume list-owned selection is the canonical truth rather than one possible local presentation of shared state

### Recommendation

Document `ListMachine` as a **local-first navigable list control** whose selection semantics may be either local or reflected into shared state by parent machines.

### Follow-up

- consider future support for shared-selected-item inputs via context/shared-backed wrappers rather than rewriting the machine model
- keep row/item IDs stable for presence overlays and remote markers

## 3. `ListDetailMachine`

### Current public shape

- composed from `ListMachine`
- local state:
  - nested `list`
  - `committed`
- `Shared = ()`

### Collaboration assessment

#### Local ephemeral state

- local list focus and navigation state
- local scroll state inherited from child list

#### Potentially shared state

- `committed` may be shared depending on application semantics
- selected/committed detail target may represent shared workspace state in downstream apps

#### Participant-local published state

Potential future publishable state:

- currently selected item as participant presence
- committed/inspected item as participant attention marker

### Risk level

**Medium**

This machine is compositionally sound, but because it models “selected versus committed” locally, it can be misread as a final answer for collaborative inspection workflows.

### Good signs

- uses machine composition rather than bespoke logic
- selection and commit are already semantically distinct
- detail projection can be driven from upstream semantics later

### Risks

- local `committed` can be mistaken for universally shared selection state
- `Shared = ()` may hide the eventual need for shared workspace/inspection state

### Recommendation

Keep `ListDetailMachine` as a demo/composition machine, but describe it as a **local-first composition example**, not a final shared-workspace primitive.

### Follow-up

- later add an example or variant where committed detail identity comes from shared state

## 4. `modal` helper

### Current public shape

- reusable helper for modal scene wrapping and modal key handling
- focus trapping and primary focus redirection
- backdrop dismissal

### Collaboration assessment

#### Local ephemeral state

- modal focus is participant-local
- tab traversal within modal is participant-local
- trap behavior is participant-local

#### Potentially shared state

- modal open/closed state may be local or shared depending on application semantics

#### Participant-local published state

Possible future published state:

- participant is interacting with modal X
- participant’s local dialog focus target

### Risk level

**Low**

This helper is already close to the right shape because it expresses modal behavior structurally and leaves open/closed ownership to the parent machine.

### Good signs

- focus behavior is local runtime behavior, which is correct
- backdrop and dialog identities are explicit
- modal wrapper does not force modal open-state ownership into a global system

### Risks

- current helper assumes one local focus order and one local focus state, which is okay now but should remain framed as participant-local

### Recommendation

Keep modal semantics participant-local by default.

When collaboration overlays arrive, treat remote modal activity as presence/annotation rather than shared focus theft.

### Follow-up

- ensure upcoming focus scope policies document participant-local semantics explicitly

## 5. `CommandPaletteMachine`

### Current public shape

- local state:
  - `open`
  - nested `input`
  - nested `list`
  - `filtered`
  - `committed`
- composed modal + input + list
- `Shared = ()`

### Collaboration assessment

#### Local ephemeral state

- local focus between input and list
- local filtering mechanics
- local input cursor/edit state
- local modal traversal

#### Potentially shared state

Depending on the application:

- `open` may be local or shared
- `committed` may represent shared command execution or only local command choice
- command registry/items may eventually come from shared state or shared capabilities upstream

#### Participant-local published state

Potential future published state:

- active participant command palette usage
- current query as presence, if an app wants to expose it
- current selection/attention within the palette

### Risk level

**Medium**

The command palette is a strong composition example, but it is currently a local-first control with local filtering and local committed state.

### Good signs

- scope-aware focus behavior is structural
- composition boundaries are clean
- child machines remain reusable
- modal state and list/input behavior can be reasoned about separately

### Risks

- `filtered` being local is fine, but it must not imply that command indexing or semantic command state is necessarily local
- `committed` may be interpreted too narrowly as local-only
- `open` state needs explicit documentation as application-defined local or shared semantics

### Recommendation

Describe `CommandPaletteMachine` as a **local-first command surface** whose semantic outcomes can be lifted into shared application state by parent machines.

### Follow-up

- document which state is intentionally local-first
- consider a later shared-command-registry example

## Cross-cutting findings

## Stable identity

This is currently a strong area.

Positive findings:

- standard machines use explicit node IDs
- list and modal helpers expose structured identity roots
- command palette composes stable IDs across children and overlay structure

Implication:

This is good groundwork for collaboration overlays such as:

- remote attention markers
- remote cursor anchors
- participant badges on surfaces
- shared annotation attachment

## Focus semantics

This is also directionally strong.

Positive findings:

- focus lives in `Runtime`, not in shared state
- focus scopes are structural in the scene
- runtime tab traversal is local and scope-aware

Implication:

This aligns well with participant-local focus semantics.

Main requirement going forward:

- do not retrofit upcoming focus policy work in a way that implies one globally authoritative focus owner

## Shared-state usage in standard machines

This is currently the weakest collaboration area.

Observation:

- all current standard machines use `Shared = ()`

This is not inherently wrong. But it means collaboration-readiness depends on documentation and disciplined future API evolution.

Recommendation:

- avoid describing current standard machines as if their local model shapes are the final collaborative abstractions
- when practical, introduce examples or variants that demonstrate shared-backed composition at the parent-machine level

## Audit conclusions

### Ready enough for `0.1.0`

- modal helper
- list machine
- command palette as a local-first composed control
- current local runtime focus model

### Needs careful framing in docs

- input machine
- list-detail machine
- command palette open/commit semantics

### Needs future design extension, not immediate rewrite

- collaborative editor abstraction beyond `InputMachine`
- shared-selected/committed patterns in standard-machine compositions
- participant-local published overlays such as remote cursors and attention markers

## Action items

## Immediate

1. keep focus scope policy work explicitly participant-local
2. add documentation notes to standard-machine docs/examples describing local-first versus shared-capable semantics
3. avoid any API changes that collapse `Model` and `Shared` responsibilities

## Near-term

4. write a short design note on participant-local presence overlays
5. build at least one example showing parent-managed semantic state lifted above local control state
6. consider shared-backed composition examples after the next round of standard machines lands

## Recommendation to proceed

Proceed with the next planned work in this order:

1. focus scope policies
2. participant-local presence overlay design note
3. continued standard-machine expansion with collaboration-readiness checks applied during review
