# Capability Gating

Multi-participant sessions need more than presence — they need policy:
who may activate what. Knopper's answer, behind the **`collaboration`**
feature, is capability gating powered by
[Schubert calculus](https://crates.io/crates/schubert): capabilities are
Schubert conditions on a Grassmannian, grants position principals, and
access checks *intersect the geometry*.

```toml
[dependencies]
knopper = { version = "0.1", features = ["collaboration"] }
```

## The payoff: impossible combinations

A set-membership ACL answers "does the principal hold X?" — and silently
allows a principal who holds two capabilities that *cannot coexist*.
Geometry knows better. The worked example from the test suite:

```rust,ignore
use knopper::capability::{AccessController, Capability, CapabilityKind};
use knopper::collaboration::ParticipantId;

// Gr(2,4): review = σ₂, deploy = σ₁₁ — σ₂·σ₁₁ = 0.
let mut acl = AccessController::new(2, 4)?;
acl.register_capability(Capability::new("review", "Approve", vec![2], CapabilityKind::ReadLike))?;
acl.register_capability(Capability::new("deploy", "Ship", vec![1, 1], CapabilityKind::WriteLike))?;

let alice = acl.create_principal("alice")?;
acl.grant(&alice, "review")?;
acl.grant(&alice, "deploy")?;   // both granted — set-membership would allow

let decision = acl.check(&alice, &["review", "deploy"])?;
// AccessDecision::Impossible { conflicting } — separation of duties
// detected by the algebra, not by a hand-written exclusion rule.
```

Separation of duties stops being a policy you must remember to write and
becomes a geometric fact the checker derives.

## `CapabilityGate`: declare the policy

A gate wraps the controller, maps semantic `NodeId`s to required
capability lists, and is bridged from a collaboration `ParticipantId`
(grant-then-gate and gate-then-grant both work). Only
`AccessDecision::Granted` opens a node; underconstrained policies and
controller errors **fail closed**.

```rust,ignore
let mut gate = CapabilityGate::new(acl, &ParticipantId::new("alice"))?;
gate.gate_node(deploy_button_id, vec!["review".into(), "deploy".into()]);
```

`gated_scene(&scene)` projects not-granted gated nodes as `disabled` —
which means focus collection and activation skip them through the same
principled path used for ordinary disabled nodes. No new gating
machinery exists inside render or focus.

## `CapabilityRuntime`: enforce at activation

`CapabilityRuntime` wraps a `Runtime` (it derefs to the plain runtime)
and consults the gate on `Activate` events. Not-granted activations are
suppressed — the machine never sees them — and the most recent decision
is available for host-side rendering:

```rust,ignore
runtime.dispatch(RuntimeEvent::Activate(deploy_button_id));
assert!(matches!(runtime.last_denial(), Some(AccessDecision::Impossible { .. })));
```

With no gate installed, `CapabilityRuntime` behaves exactly like the
plain runtime. The feature is off by default; the default build pays
nothing for it.

## Layering

The capability seam honors the geometric substrate's boundary (see
[The Geometric Substrate](../concepts/geometric-substrate.md)): GA3
stays the fingerprint layer; this seam consumes collaboration identity
at its edge and runs the higher-grade arithmetic in Schubert/amari
territory. Nothing pushes its algebra back into GA3.
