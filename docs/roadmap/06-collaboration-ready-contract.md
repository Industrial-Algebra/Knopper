# Knopper collaboration-ready contract for 0.1.0

> **Roadmap document** — Knopper 0.1.0
>
> Turns the conceptual collaboration-readiness guide
> ([01-collaboration-readiness.md](01-collaboration-readiness.md)) and the
> standard-machine audit ([02](02-standard-machine-collaboration-audit.md))
> into a **concrete, code-grounded contract** that downstream IA projects
> (Wallace, Dominic, Tsume) can consume today to build functional multi-user
> TUIs on top of Knopper.
>
> It also corrects the Schubert integration sketch in
> [05-distributed-correctness-schubert.md](05-distributed-correctness-schubert.md)
> against the actual Schubert 0.3.0 public API.

## Why this document exists

The earlier collaboration docs established the right *taxonomy*
(Model / Shared / Participant-local / Derived) and audited the standard
machines conceptually. But for downstream projects to consume Knopper **in
functional multi-user form**, they need:

1. a concrete `Shared` payload type they can populate,
2. a documented merge seam on `Runtime`,
3. participant identity and presence types they can project from,
4. a verified mapping to Schubert's real API for the future capability layer,
5. an explicit statement of what is and is not supported in `0.1.0`.

This document specifies each of those, and the `collaboration` module
(`src/collaboration.rs`) is its executable counterpart.

## What is supported in 0.1.0

| Capability | Status | Where |
|---|---|---|
| Local participant runtime with participant-local focus | ✅ Supported | `Runtime`, `FocusState` |
| Semantic `Shared` lane on every `Machine` | ✅ Supported | `Machine::Shared`, `Machine::project` |
| External shared-state merge seam | ✅ Supported | `Runtime::set_shared` |
| Participant identity + presence + roster types | ✅ Supported | `collaboration::{ParticipantId, Presence, ParticipantRoster}` |
| Presence anchors in the scene | ✅ Supported | `Annotation::{PresenceSlot, RemoteCursor}` |
| Single-local-cursor rendering | ✅ Supported | `Machine::cursor_position` |
| Remote presence derivation in projection | ✅ Supported (manual) | parent `project` reads roster, emits overlays |
| Distribution / CRDT merge / network transport | ❌ Out of scope 0.1.0 | downstream-owned |
| Schubert capability checking | ❌ Out of scope 0.1.0 | planned 0.2.0, see §5 |
| Multi-cursor input handling per key | ❌ Out of scope 0.1.0 | one local cursor; remotes are overlays |

## 1. The participant runtime model

Knopper's collaboration model is **one `Runtime` per participant**.

- Each participant (human or AI agent) drives their own `Runtime`.
- `Runtime` owns **participant-local** state only: `FocusState`, the local
  cursor, renderer/backend caches, last render ops.
- The `Shared` lane carries the **replicated semantic world** everyone is
  looking at, plus a `ParticipantRoster` describing who is present.
- Each runtime renders its own terminal. Collaboration synchronizes the
  semantic `Shared` state, never terminal byte streams (per guide §6 of
  [01]).

This is why `Runtime::cursor()` returns a single cursor — it is the *local*
participant's cursor. Remote cursors arrive as scene annotations derived
from the roster during projection.

## 2. The `Shared` payload contract

The canonical `Shared` payload for a collaborative machine is a struct that
contains, at minimum:

```rust
use knopper::ParticipantRoster;

/// Example collaborative Shared payload for a downstream app.
pub struct WorkspaceShared {
    /// Who is in the session and what they're attending to.
    pub roster: ParticipantRoster,
    /// The replicated semantic world (documents, tasks, annotations, ...).
    pub workspace: Workspace,
}
```

`ParticipantRoster` (in `src/collaboration.rs`) provides:

- `ParticipantRoster::with_local(id)` — root the roster on this runtime's
  local participant.
- `roster.local()` / `roster.local_mut()` — the local participant's entry.
- `roster.remotes()` — iterate over everyone else.
- `roster.upsert(presence)` / `roster.remove(id)` — maintain the roster as
  remote updates arrive.
- The local participant's tone is forced to `PresenceTone::Local` on upsert,
  so the local/remote distinction cannot drift.

`Presence` carries `id`, `label`, `tone` (`Local` / `Collaborator` /
`Passive`), and an optional semantic `anchor: Option<NodeId>` for attaching
overlays to stable scene surfaces.

## 3. The merge seam

`Runtime::set_shared(&self, shared)` is the convergence seam. It is
deliberately **external-merge**: Knopper does not own the distribution
layer in `0.1.0`. The intended flow:

```
[remote update arrives]
        │
        ▼
downstream convergence layer (CRDT, server fan-out, ...)
        │  merges remote roster/workspace into local Shared
        ▼
runtime.set_shared(merged_shared)
        │
        ▼
Behavior<Shared> notifies projection → scene reprojections → re-render
```

This works because `Shared` flows through a `cliffy_core::Behavior`, so
`set_shared` triggers reactive reprojection of the scene automatically. No
render-loop changes are required downstream.

**What downstream must NOT do:** mutate `Shared` from inside a `Machine`
update and expect remote propagation. Updates flow *into* `Shared` via the
convergence layer; `Machine::update` may only propose changes (via effects
or by writing through a shared handle the convergence layer owns).

## 4. Deriving presence overlays in projection

A collaborative parent machine's `project` reads the roster out of `Shared`
and emits presence overlays as scene structure. The existing
`Annotation::{PresenceSlot(String), RemoteCursor(NodeId)}` variants are the
anchor points; rendering overlays against them is the parent machine's job
(or a future helper).

The in-tree `demo_ui::presence_strip` and `PresenceCue` are the
presentation-layer projection of a `collaboration::Presence`: they map
semantic tone to color. Downstream apps can follow the same shape —
semantic `Presence` in `Shared`, presentation cues in projection.

### Example skeleton

```rust
fn project(model: &Model, shared: &WorkspaceShared, ctx: &Ctx) -> Scene<Msg> {
    let main_workspace = project_workspace(&model.view, &shared.workspace, ctx);
    let presence_bar = presence_strip(base, &shared.roster.remotes().cloned().collect::<Vec<_>>());
    Scene::column(root, vec![main_workspace, presence_bar])
}
```

## 5. Schubert integration — corrected against the 0.3.0 API

The earlier sketch in [05] used approximate signatures. Against the actual
Schubert 0.3.0 release, the integration maps as follows. **This remains
planned for Knopper 0.2.0 behind a `collaboration` feature; it is documented
here so downstream projects can plan.**

### Relevant Schubert 0.3.0 types

| Schubert type | Purpose |
|---|---|
| `AccessController` | Synchronous capability store; `new(k,n)`, `register_capability`, `create_principal`, `grant`, `revoke`, `check(&PrincipalId, &[&str]) -> Result<AccessDecision>` |
| `Capability::new(id, desc, codim_partitions: Vec<usize>, CapabilityKind)` | A Schubert condition on the Grassmannian |
| `AccessDecision` | `Granted{configurations, path}` / `Impossible{conflicting}` / `Denied` / `Underconstrained{dimension}` |
| `crdt::CrdtState` | Eventually-consistent grant store: `grant(principal, cap, node, ts)`, `merge(&other)`, `check`, `version()`, `set_max_staleness`, `staleness_ms` |
| `VersionVector` | Vector clock with `happens_before`, `merge` |
| `MultiController` | Multi-tenant: multiple Grassmannian domains |
| `AccessContext` | resource/time/metadata for temporal and scoped checks |
| `policy` (TOML) | Declarative policy definition |

### Planned Knopper seam (0.2.0)

```toml
[features]
collaboration = ["dep:schubert"]

[dependencies]
schubert = { version = "0.3", optional = true }
```

```rust
pub struct CapabilityRuntime<M> {
    runtime: Runtime<M>,
    #[cfg(feature = "collaboration")]
    acl: Option<schubert::AccessController>,
}

#[cfg(feature = "collaboration")]
impl<M> CapabilityRuntime<M> {
    /// Check whether the local participant may perform an action on shared
    /// state. Schubert catches impossible combinations boolean ACLs miss.
    pub fn check_capability(
        &self,
        principal: &schubert::PrincipalId,
        required: &[&str],
    ) -> Result<schubert::AccessDecision, schubert::SchubertError> {
        match &self.acl {
            Some(acl) => acl.check(principal, required),
            None => Ok(schubert::AccessDecision::Granted {
                configurations: 1,
                path: schubert::ComputationPath::LittlewoodRichardson,
            }),
        }
    }
}
```

The mapping from the game-sync design
([Schubert docs/design/distributed-game-sync.md](https://github.com/Industrial-Algebra/Schubert/blob/develop/docs/design/distributed-game-sync.md))
to a coding harness is unchanged from [05]: `σ₂·σ₁₁ = 0` (write + self-review)
becomes a real `AccessDecision::Impossible` rather than a heuristic conflict.

## 6. Single-user assumptions to avoid baking in

The 0.1.0 audit found these assumptions present or tempting. Each is
acceptable *for now* but must not harden:

1. **`Shared = ()` on standard machines.** Fine as a local-first
   simplification. Do not document these machines as collaborative
   primitives. The new `collaboration` module gives downstream a real
   `Shared` payload to wrap them with.
2. **Single cursor in `Machine::cursor_position`.** This is the local
   participant's cursor. Remote cursors must be overlays, not extra return
   values.
3. **`Runtime` owns one `FocusState`.** Correct — focus is
   participant-local. Do not add a "global focus" concept.
4. **Selection/commit living in `Model` on list/list-detail/command-palette.**
   Acceptable local-first. Parent machines that want shared selection should
   source it from `Shared` and feed it down via context, not rewrite the
   machine.
5. **`set_shared` is wholesale replace, not incremental merge.** Intentional
   for 0.1.0: downstream owns merge. A future CRDT-backed `Shared` wrapper
   (Phase 2 in [05]) can make this incremental without changing the seam.

## 7. Minimum contract for downstream IA projects

Downstream projects (Wallace, Dominic, Tsume) can rely on the following in
Knopper 0.1.0:

- **One runtime per participant.** Spin up a `Runtime` per human/agent.
- **`Shared` carries the replicated world plus a `ParticipantRoster`.**
  Downstream owns the convergence layer (CRDT, server) and pushes merged
  state via `Runtime::set_shared`.
- **Focus is participant-local.** Each runtime's `FocusState` is its own;
  remote focus is never imposed locally.
- **Presence overlays derive from the roster in projection.** Use
  `Annotation::{PresenceSlot, RemoteCursor}` as anchors.
- **Node IDs are stable and semantic.** Overlays and remote anchors attach
  to stable IDs, not render-time positions.
- **The Schubert capability seam arrives in 0.2.0** behind a feature flag
  and does not disturb the 0.1.0 API surface.

## 8. Verification checklist for new machines and runtime features

Before adding any machine or runtime feature, confirm:

- [ ] Is its state local, shared, participant-local-published, or derived?
- [ ] Does it assume a single globally authoritative focus or selection?
- [ ] Is there a clean seam for a remote update to reach it via `Shared`?
- [ ] Are its `NodeId`s stable across reprojection for overlay anchoring?
- [ ] Does the API documentation avoid implying single-user ownership where
      collaborative use is intended?

---

*Roadmap document for Knopper 0.1.0. The executable counterpart is
`src/collaboration.rs`. References Schubert 0.3.0.*
