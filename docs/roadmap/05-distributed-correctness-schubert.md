# Distributed Correctness via Schubert Integration

> **Roadmap Document** — Knopper post-0.1.0
>
> Positions Schubert as the distributed state correctness layer for
> Knopper's multi-user collaborative runtime. Draws on the distributed
> game sync use case (Schubert `docs/design/distributed-game-sync.md`)
> as the motivating formal model.

## Motivation

Knopper is designed from the ground up for multi-user collaboration. The
collaboration-readiness guide ([01-collaboration-readiness.md](01-collaboration-readiness.md))
establishes the `Model` / `Shared` / `Participant-local` / `Derived` state
taxonomy and the invariant that shared semantic state must converge.

But convergence alone is not enough. When multiple participants — humans
and AI agents — edit shared state simultaneously, the system must also
detect **impossible state combinations**: edits that are individually
valid but collectively contradictory. Boolean merge logic (check each
edit independently → approve) misses emergent conflicts. Schubert's
geometric intersection catches them.

This document defines how Schubert becomes Knopper's correctness layer
for distributed collaborative state.

## The Multi-User AI Coding Harness Vision

Knopper's long-term goal: the foundation of a multi-user AI coding
harness where:

- Multiple AI agents (Claude, GPT, local models) work on the same codebase
- Human developers participate in the same session
- Each contributor has different capabilities (read, write, review, merge, deploy)
- State is distributed across contributors' machines (eventually consistent)
- **Correctness across distributed contributors is the central concern**

### The Problem

In a multi-agent coding session:

1. **Agent A** edits `src/auth.rs` (has write capability)
2. **Agent B** simultaneously edits `src/auth.rs` (also has write capability)
3. **Human C** approves a merge (has review capability)
4. **Agent D** deploys to staging (has deploy capability)

Each action is individually valid. But certain combinations are impossible:

- Agent D deploys code that Agent C hasn't reviewed (deploy + unreviewed = impossible)
- Agent A and Agent B both write the same function (concurrent write conflict)
- Agent B reviews its own code (write + self-review = separation-of-duty violation)

A boolean system checks each capability independently:
```
Does Agent D have deploy capability? → YES
→ APPROVE
```

It misses the geometric interaction: deploy and unreviewed states are
incompatible *together* even though each is individually valid.

### The Schubert Solution

Map each contributor's session state to a Grassmannian:

| Schubert Condition | Coding Session Meaning |
|---|---|
| σ₁ (codim 1) | Can read files |
| σ₂ (codim 2) | Can write files |
| σ₁₁ (codim 2) | Can review PRs (different geometric direction from σ₂) |
| σ₂₁ (codim 3) | Can merge (requires write + review) |
| σ₂₂ (codim 4) | Can deploy (point class — maximal constraint) |

**Impossibility detection for coding sessions:**

- σ₂ · σ₁₁ = 0: Can't simultaneously write AND review the same code
  (separation of duty — the same subspace can't satisfy both)
- σ₂ · σ₂ = 0: Can't have two concurrent writers on the same file region
  (write-write conflict detected geometrically)
- σ₂₂ · σ_unreviewed = 0: Can't deploy code that hasn't passed review

**Configuration count for collaboration:**

- Intersection = 0: **Impossible collaboration state** — reject the merge
- Intersection = 1: **Unique merge** — deterministic resolution
- Intersection > 1: **Multiple valid resolutions** — emergent collaboration
  (e.g., two valid ways to merge non-conflicting changes)

## How Schubert Maps to Knopper's State Taxonomy

The collaboration-readiness guide defines four state buckets. Schubert
provides the formal model for each:

### A. Local Ephemeral State (Model)

Not checked by Schubert. This is purely participant-local:
- Local focus path
- Local cursor position
- Local viewport

### B. Shared Replicated State (Shared)

**This is where Schubert intersection applies.** When Knopper's `Shared`
state is updated by multiple participants:

1. Each participant's proposed state update is a Schubert condition
2. The server computes the intersection of all conditions
3. Zero intersection → reject (impossible state combination)
4. Positive intersection → merge with the multiplicity telling you how
   many valid reconciliations exist

```rust
// Knopper runtime integration sketch
use schubert::{AccessController, AccessDecision, Capability, CapabilityKind};

struct CollaborativeRuntime {
    /// Knopper's shared state, guarded by Schubert
    acl: AccessController,
    /// CRDT for distributed state propagation
    crdt: schubert::crdt::CrdtState,
}

impl CollaborativeRuntime {
    /// Attempt to apply a shared-state update from a participant.
    /// Returns the Schubert decision — impossible states are rejected.
    fn apply_shared_update(
        &mut self,
        participant: &str,
        capabilities: &[&str],
    ) -> AccessDecision {
        let pid = schubert::PrincipalId::new(participant);
        self.acl.check(&pid, capabilities)
            .unwrap_or(AccessDecision::Denied)
    }
}
```

### C. Participant-Local Published State

Checked for geometric compatibility before publication:
- Remote cursor positions must be geometrically compatible with the
  shared state (a cursor can't anchor to a position that doesn't exist
  in the merged view)
- Presence metadata (online/offline/editing) interacts with capabilities
  (an offline participant can't hold a merge lock)

### D. Derived Presentation State

Computed from the intersection result:
- **Divergence markers**: Show when local state diverges from merged state
- **Conflict indicators**: Show when an intersection returned zero
- **Multiplicity badges**: Show when multiple valid reconciliations exist
  ("2 possible merges — click to choose")

## The CRDT Connection

Knopper's collaboration model requires eventually-consistent state.
Schubert's CRDT module provides exactly this:

- **Version vectors** for causal ordering across participants
- **Last-write-wins** with timestamp tiebreaking for concurrent edits
- **Commutative, associative, idempotent merge** — the three CRDT properties
- **Staleness gating** (v0.2.0): reject merges when state is too stale

The boundary discipline from Schubert's architectural philosophy applies
directly to Knopper:

- **The merge math must be exact.** Schubert intersection numbers are
  integers. If the intersection is zero, no amount of CRDT convergence
  will make the state valid.
- **The state distribution may be approximate.** Participants may have
  stale views. The CRDT resolves this eventually.

## Implementation Phasing

### Phase 1: Capability-Aware Runtime (Knopper 0.2.0)

Add `schubert` as an optional dependency behind a `collaboration` feature:

```toml
[features]
collaboration = ["dep:schubert"]

[dependencies]
schubert = { version = "0.3", optional = true }
```

The runtime gains a capability check seam:

```rust
pub struct Runtime {
    // ... existing fields ...
    #[cfg(feature = "collaboration")]
    acl: Option<schubert::AccessController>,
}

impl Runtime {
    /// Check if a participant can perform an action on shared state.
    #[cfg(feature = "collaboration")]
    pub fn check_capability(
        &self,
        participant: &str,
        required: &[&str],
    ) -> CapabilityResult {
        match &self.acl {
            Some(acl) => match acl.check(&PrincipalId::new(participant), required) {
                Ok(AccessDecision::Granted { configurations }) => {
                    CapabilityResult::Allowed { configurations }
                }
                Ok(AccessDecision::Impossible { conflicting }) => {
                    CapabilityResult::ImpossibleConflict(conflicting)
                }
                _ => CapabilityResult::Denied,
            },
            None => CapabilityResult::NoAclConfigured,
        }
    }
}
```

### Phase 2: CRDT-Backed Shared State (Knopper 0.3.0)

Replace the `set_shared()` seam with a CRDT-backed shared state:

```rust
#[cfg(feature = "collaboration")]
pub struct CollaborativeShared<S> {
    inner: S,
    crdt: schubert::crdt::CrdtState,
}

impl<S: Clone> CollaborativeShared<S> {
    /// Merge remote state updates with geometric impossibility detection.
    pub fn merge_remote(&mut self, remote: &CrdtState) -> MergeResult {
        // CRDT merge (eventually consistent)
        self.crdt.merge(remote);

        // Staleness check
        if let Some(staleness) = self.crdt.staleness_ms() {
            if staleness > self.max_staleness {
                return MergeResult::Stale(staleness);
            }
        }

        // Geometric impossibility check
        // (Application-specific: extract capabilities from state and check)
        MergeResult::Merged
    }
}
```

### Phase 3: Multi-Agent Coding Harness (Knopper 0.4.0+)

Full multi-agent collaboration:

- Multiple AI agents register as principals with different capabilities
- Each agent's edits are Schubert conditions on the shared code state
- The harness detects impossible edit combinations (write-write conflicts,
  separation-of-duty violations, deploy-without-review)
- The CRDT propagates state across agents' machines
- Schubert staleness gating prevents stale agents from corrupting state

## Dependency Justification

The collaboration-readiness guide asks downstream projects to rely on
Knopper's state separation. Schubert makes that separation **enforced**
rather than conventional:

| Without Schubert | With Schubert |
|---|---|
| Shared state converges via CRDT | Shared state converges AND is geometrically valid |
| Impossible states silently merge | Impossible states detected and rejected |
| "Best effort" collaboration | Mathematically guaranteed collaboration |
| Conflicts found by diff heuristics | Conflicts found by intersection numbers |

## Connection to the Game Sync Design

The distributed game sync use case
([Schubert docs/design/distributed-game-sync.md](https://github.com/Industrial-Algebra/Schubert/blob/develop/docs/design/distributed-game-sync.md))
provides the formal model. The mapping from game sync to coding harness:

| Game Sync | Coding Harness |
|---|---|
| Player state (position, velocity, health) | Session state (files, cursor, selections) |
| Game rules (combat mode, safe zone) | Capabilities (write, review, deploy) |
| σ₂ · σ₁₁ = 0 (combat + safe zone) | σ₂ · σ₁₁ = 0 (write + self-review) |
| Happy accident (multiplicity > 1) | Emergent collaboration (multiple valid merges) |
| Desync detection (zero intersection) | Conflict detection (zero intersection) |
| CRDT for player state propagation | CRDT for code state propagation |

The formal mapping, operational definitions, and impossibility proofs from
the game sync document transfer directly to the coding harness context.

## Summary

Schubert provides what Knopper's collaboration model needs: a formal
correctness layer for distributed state that catches impossible
combinations boolean systems miss. The integration is phased —
capability-aware runtime first, CRDT-backed shared state second,
full multi-agent harness third — and each phase adds a mathematical
guarantee that was previously only a convention.

The game sync design document proves the formal model works. The coding
harness is its first real-world application.

---

*Roadmap document for Knopper. References Schubert v0.3.0 design:
`docs/design/distributed-game-sync.md`.*
