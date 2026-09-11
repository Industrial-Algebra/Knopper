// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Schubert capability seam (feature `collaboration`; identity-restoration
//! plan Unit 3, roadmap 06 §5 pulled forward).
//!
//! Capability-gated interaction for multi-participant sessions, powered by
//! [Schubert calculus](https://crates.io/crates/schubert): capabilities are
//! Schubert conditions on a Grassmannian, grants position principals, and
//! `check` intersects the geometry. The payoff over set-membership ACLs is
//! **impossible-combination detection**: separation-of-duties constraints
//! become geometric facts (σ₂·σ₁₁ = 0 in Gr(2,4)), rejected with
//! [`AccessDecision::Impossible`] instead of silently allowed or opaquely
//! denied.
//!
//! Two layers:
//!
//! - [`CapabilityGate`] — the declarative half: a Schubert
//!   [`AccessController`] plus a map of semantic [`NodeId`]s to required
//!   capability lists, bridged from collaboration's [`ParticipantId`].
//!   It answers per-node decisions and projects gated scenes (not-granted
//!   gated nodes render as `disabled`, so focus collection and activation
//!   skip them through the existing principled path).
//! - [`CapabilityRuntime`] — the runtime half: wraps [`Runtime`],
//!   consulting the gate on [`RuntimeEvent::Activate`] and suppressing
//!   activations that are not [`AccessDecision::Granted`] (fail-closed).
//!
//! Boundary note (encoding contract §7): GA3 stays the fingerprint layer;
//! this seam consumes collaboration identity at its boundary and runs the
//! higher-grade arithmetic in Schubert/amari territory. It never pushes its
//! algebra back into GA3.

use std::collections::HashMap;

use crate::RuntimeEvent;
use crate::collaboration::ParticipantId;
use crate::id::NodeId;
use crate::runtime::Runtime;
use crate::scene::Scene;
// Re-exported for hosts building controllers: the seam is typed in terms
// of Schubert's own surface (phantom-free by upstream design).
use schubert::{AccessController, AccessDecision};
pub use schubert::{Capability, CapabilityKind, PrincipalId};

/// The declarative half of the Schubert capability seam: a Schubert
/// [`AccessController`] (the geometry) plus a registry of semantic
/// [`NodeId`]s → required capability lists (the policy), evaluated for one
/// bridged [`ParticipantId`].
///
/// Decisions are Schubert's own: a gated node is *open* only under
/// [`AccessDecision::Granted`]. `Underconstrained` (requirements don't pin
/// the geometry) and controller errors (unknown capability, unknown
/// principal) both **fail closed**. The interesting rejection is
/// [`AccessDecision::Impossible`]: the principal holds every required
/// capability, and the algebra proves they cannot coexist
/// (e.g. σ₂·σ₁₁ = 0 — separation of duties as a geometric fact).
pub struct CapabilityGate {
    controller: AccessController,
    principal: PrincipalId,
    gates: HashMap<NodeId, Vec<String>>,
}

impl CapabilityGate {
    /// Build a gate over a controller, acting as `participant`.
    ///
    /// The participant is bridged verbatim from the collaboration
    /// [`ParticipantId`] and registered as a Schubert principal if not
    /// already known (hosts that grant before gating will have registered
    /// it already; both orders work).
    ///
    /// # Errors
    /// Returns the controller's error if principal registration fails.
    pub fn new(
        mut controller: AccessController,
        participant: &ParticipantId,
    ) -> Result<Self, schubert::SchubertError> {
        let principal = match controller.create_principal(participant.as_str()) {
            Ok(principal) => principal,
            Err(schubert::SchubertError::PrincipalExists(_)) => {
                PrincipalId::new(participant.as_str())
            }
            Err(e) => return Err(e),
        };
        Ok(Self {
            controller,
            principal,
            gates: HashMap::new(),
        })
    }

    /// Require `required` capabilities to activate `node`.
    pub fn gate_node(&mut self, node: NodeId, required: Vec<String>) -> &mut Self {
        self.gates.insert(node, required);
        self
    }

    /// The Grassmannian parameters `Gr(k,n)` the controller reasons over.
    #[must_use]
    pub fn grassmannian(&self) -> (usize, usize) {
        self.controller.grassmannian()
    }

    /// Whether `node` carries capability requirements.
    #[must_use]
    pub fn is_gated(&self, node: &NodeId) -> bool {
        self.gates.contains_key(node)
    }

    /// The Schubert decision for this principal activating `node`.
    ///
    /// `Ok(None)` = ungated (nothing to check). Errors fail closed at the
    /// callers ([`Self::is_granted`]); they surface here for hosts that
    /// want them.
    ///
    /// # Errors
    /// Propagates controller errors (unknown capability id, ...).
    pub fn decision(
        &self,
        node: &NodeId,
    ) -> Result<Option<AccessDecision>, schubert::SchubertError> {
        let Some(required) = self.gates.get(node) else {
            return Ok(None);
        };
        let refs: Vec<&str> = required.iter().map(String::as_str).collect();
        Ok(Some(self.controller.check(&self.principal, &refs)?))
    }

    /// Whether activation of `node` is open for this principal.
    ///
    /// Fail-closed: only [`AccessDecision::Granted`] opens; ungated nodes
    /// are open; errors close.
    #[must_use]
    pub fn is_granted(&self, node: &NodeId) -> bool {
        matches!(
            self.decision(node),
            Ok(Some(AccessDecision::Granted { .. }))
        )
    }

    /// Project `scene` for this principal: gated nodes that are not
    /// granted render as `disabled`, so focus collection and activation
    /// skip them through the existing principled path. Ungated and granted
    /// nodes pass through unchanged.
    #[must_use]
    pub fn gated_scene<Msg: Clone>(&self, scene: &Scene<Msg>) -> Scene<Msg> {
        self.project(scene)
    }

    fn project<Msg: Clone>(&self, scene: &Scene<Msg>) -> Scene<Msg> {
        let gated_here = |meta: &crate::scene::NodeMeta| {
            let id = meta.id;
            self.is_gated(&id) && !self.is_granted(&id)
        };
        match scene {
            Scene::Empty => Scene::Empty,
            Scene::Text(text) => {
                let mut rebuilt = text.clone();
                if gated_here(&rebuilt.meta) {
                    rebuilt.meta.disabled = true;
                }
                Scene::Text(rebuilt)
            }
            Scene::Row { meta, children } => Scene::Row {
                meta: self.disabled_if(meta, gated_here),
                children: children.iter().map(|c| self.project(c)).collect(),
            },
            Scene::Column { meta, children } => Scene::Column {
                meta: self.disabled_if(meta, gated_here),
                children: children.iter().map(|c| self.project(c)).collect(),
            },
            Scene::Stack { meta, children } => Scene::Stack {
                meta: self.disabled_if(meta, gated_here),
                children: children.iter().map(|c| self.project(c)).collect(),
            },
            Scene::FocusScope {
                meta,
                name,
                policy,
                child,
            } => Scene::FocusScope {
                meta: self.disabled_if(meta, gated_here),
                name: name.clone(),
                policy: *policy,
                child: Box::new(self.project(child)),
            },
            Scene::Align {
                meta,
                anchor,
                child,
            } => Scene::Align {
                meta: self.disabled_if(meta, gated_here),
                anchor: *anchor,
                child: Box::new(self.project(child)),
            },
            Scene::Padding {
                meta,
                padding,
                child,
            } => Scene::Padding {
                meta: self.disabled_if(meta, gated_here),
                padding: *padding,
                child: Box::new(self.project(child)),
            },
            Scene::Sized {
                meta,
                constraint,
                child,
            } => Scene::Sized {
                meta: self.disabled_if(meta, gated_here),
                constraint: *constraint,
                child: Box::new(self.project(child)),
            },
            Scene::Viewport { meta, child } => Scene::Viewport {
                meta: self.disabled_if(meta, gated_here),
                child: Box::new(self.project(child)),
            },
            Scene::Scroll {
                meta,
                offset,
                child,
            } => Scene::Scroll {
                meta: self.disabled_if(meta, gated_here),
                offset: *offset,
                child: Box::new(self.project(child)),
            },
            Scene::Border { meta, child } => Scene::Border {
                meta: self.disabled_if(meta, gated_here),
                child: Box::new(self.project(child)),
            },
            Scene::Annotated { meta, label, child } => Scene::Annotated {
                meta: self.disabled_if(meta, gated_here),
                label: label.clone(),
                child: Box::new(self.project(child)),
            },
        }
    }

    fn disabled_if(
        &self,
        meta: &crate::scene::NodeMeta,
        gated_here: impl Fn(&crate::scene::NodeMeta) -> bool,
    ) -> crate::scene::NodeMeta {
        let mut meta = meta.clone();
        if gated_here(&meta) {
            meta.disabled = true;
        }
        meta
    }
}

/// The runtime half of the seam: wraps a [`Runtime`], consulting an
/// optional [`CapabilityGate`] on [`RuntimeEvent::Activate`]. Activations
/// that are not granted are suppressed (fail-closed), and the most recent
/// denial is exposed via [`Self::last_denial`] for host-side rendering.
///
/// Derefs to the inner [`Runtime`] — every non-activation method is the
/// plain runtime (zero behavioral change with no gate installed).
pub struct CapabilityRuntime<M>
where
    M: crate::machine::Machine,
{
    runtime: Runtime<M>,
    gate: Option<CapabilityGate>,
    last_denial: Option<AccessDecision>,
}

impl<M> std::ops::Deref for CapabilityRuntime<M>
where
    M: crate::machine::Machine,
    M::Context: Clone,
    M::Model: Clone + crate::IntoGeometric + crate::FromGeometric + 'static,
    M::Shared: Clone + crate::IntoGeometric + crate::FromGeometric + 'static,
    M::Msg: Clone + 'static,
{
    type Target = Runtime<M>;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}

impl<M> CapabilityRuntime<M>
where
    M: crate::machine::Machine,
    M::Context: Clone,
    M::Model: Clone + crate::IntoGeometric + crate::FromGeometric + 'static,
    M::Shared: Clone + crate::IntoGeometric + crate::FromGeometric + 'static,
    M::Msg: Clone + 'static,
{
    /// Wrap a runtime with no gate: behaves exactly like the plain runtime.
    pub fn new(runtime: Runtime<M>) -> Self {
        Self {
            runtime,
            gate: None,
            last_denial: None,
        }
    }

    /// Wrap a runtime with a capability gate installed.
    pub fn with_gate(runtime: Runtime<M>, gate: CapabilityGate) -> Self {
        Self {
            runtime,
            gate: Some(gate),
            last_denial: None,
        }
    }

    /// Install or replace the gate.
    pub fn set_gate(&mut self, gate: CapabilityGate) {
        self.gate = Some(gate);
    }

    /// The installed gate, if any.
    #[must_use]
    pub fn gate(&self) -> Option<&CapabilityGate> {
        self.gate.as_ref()
    }

    /// Dispatch with capability-gated activation.
    ///
    /// [`RuntimeEvent::Activate`] on a gated, not-granted node is
    /// suppressed (the model never sees it); every other event — and
    /// ungated activations — pass through unchanged.
    pub fn dispatch(&mut self, event: RuntimeEvent) {
        if let (Some(gate), RuntimeEvent::Activate(node)) = (&self.gate, &event) {
            match gate.decision(node) {
                Ok(None) | Ok(Some(AccessDecision::Granted { .. })) => {}
                Ok(Some(denial)) => {
                    self.last_denial = Some(denial);
                    return;
                }
                Err(_) => {
                    // Fail closed: errors suppress without a decision to show.
                    self.last_denial = None;
                    return;
                }
            }
        }
        self.last_denial = None;
        self.runtime.dispatch(event);
    }

    /// The most recent suppressed activation's decision (cleared on every
    /// successful dispatch).
    #[must_use]
    pub fn last_denial(&self) -> Option<&AccessDecision> {
        self.last_denial.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RuntimeEvent;
    use crate::collaboration::ParticipantId;
    use crate::runtime::Runtime;
    use crate::scene::{Role, Scene};
    use crate::{Effect, NodeId};
    use schubert::{AccessController, AccessDecision, Capability, CapabilityKind};

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Ctx;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Msg {
        Increment,
    }

    /// A minimal hand-rolled machine: every `Activate(7)` increments the
    /// model. (Concreteness matters here — opaque closure types would hide
    /// `Clone` from `Runtime::new`'s bounds.)
    #[derive(Clone)]
    struct CountMachine;

    impl crate::machine::Machine for CountMachine {
        type Context = Ctx;
        type Msg = Msg;
        type Model = i32;
        type Shared = String;

        fn init(&self, _ctx: &Ctx) -> i32 {
            0
        }

        fn update(&self, model: &mut i32, _msg: Msg, _ctx: &Ctx) -> Effect<Msg> {
            *model += 1;
            Effect::None
        }

        fn project(
            &self,
            model: cliffy_core::Behavior<i32>,
            _shared: cliffy_core::Behavior<String>,
            _ctx: &Ctx,
        ) -> crate::machine::SceneBehavior<Msg> {
            cliffy_core::behavior(
                Scene::text(7_u64, format!("count:{}", model.sample()))
                    .with_role(Role::StatusLine)
                    .on_activate(Msg::Increment),
            )
        }
    }

    fn machine() -> CountMachine {
        CountMachine
    }

    /// Gr(2,4) with the classic separation-of-duties geometry:
    /// `review` = σ₂ (`[2]`), `deploy` = σ₁₁ (`[1,1]`) — codimensions
    /// sum to dim Gr(2,4) = 4 but σ₂·σ₁₁ = 0: no configuration satisfies
    /// both. The algebra knows they cannot coexist.
    fn duty_separated_controller() -> AccessController {
        let mut acl = AccessController::new(2, 4).expect("Gr(2,4) is valid");
        acl.register_capability(Capability::new(
            "review",
            "Approve a change",
            vec![2],
            CapabilityKind::ReadLike,
        ))
        .expect("register review");
        acl.register_capability(Capability::new(
            "deploy",
            "Ship without further approval",
            vec![1, 1],
            CapabilityKind::WriteLike,
        ))
        .expect("register deploy");
        acl.register_capability(Capability::new(
            "signoff",
            "First signoff",
            vec![2],
            CapabilityKind::ReadLike,
        ))
        .expect("register signoff");
        acl.register_capability(Capability::new(
            "countersign",
            "Second signoff",
            vec![2],
            CapabilityKind::ReadLike,
        ))
        .expect("register countersign");
        acl
    }

    #[test]
    fn schubert_detects_impossible_combination() {
        let mut acl = duty_separated_controller();
        let alice = acl.create_principal("alice").expect("create alice");
        acl.grant(&alice, "review").expect("grant review");
        acl.grant(&alice, "deploy").expect("grant deploy");

        // The principal HOLDS both — set-membership ACLs would allow.
        // Geometry says they cannot coexist: Impossible, with the pair named.
        let decision = acl.check(&alice, &["review", "deploy"]).expect("check");
        match decision {
            AccessDecision::Impossible { conflicting } => {
                let ids: Vec<String> = conflicting.iter().map(|c| c.as_str().to_string()).collect();
                assert!(ids.contains(&"review".to_string()));
                assert!(ids.contains(&"deploy".to_string()));
            }
            other => panic!("expected Impossible, got {other:?}"),
        }
    }

    #[test]
    fn gate_suppresses_activation_on_impossible_requirements() {
        let mut acl = duty_separated_controller();
        let alice = acl.create_principal("alice").expect("create alice");
        acl.grant(&alice, "review").expect("grant review");
        acl.grant(&alice, "deploy").expect("grant deploy");

        let mut gate = CapabilityGate::new(acl, &ParticipantId::new("alice")).expect("gate builds");
        gate.gate_node(
            NodeId::new(7),
            vec!["review".to_string(), "deploy".to_string()],
        );

        let mut runtime =
            CapabilityRuntime::with_gate(Runtime::new(machine(), Ctx, "shared".to_string()), gate);

        // The gated node's activation is suppressed: no model change.
        runtime.dispatch(RuntimeEvent::Activate(NodeId::new(7)));
        assert_eq!(runtime.model(), 0, "activation suppressed");
        assert!(
            matches!(
                runtime.last_denial(),
                Some(AccessDecision::Impossible { .. })
            ),
            "denial is geometric impossibility, got {:?}",
            runtime.last_denial()
        );
    }

    #[test]
    fn gate_allows_granted_activation() {
        // σ₂·σ₂ in Gr(2,4) has a nonzero LR coefficient (2 configurations):
        // two signoffs compose — activation goes through.
        let mut acl = duty_separated_controller();
        let bob = acl.create_principal("bob").expect("create bob");
        acl.grant(&bob, "signoff").expect("grant signoff");
        acl.grant(&bob, "countersign").expect("grant countersign");

        let mut gate = CapabilityGate::new(acl, &ParticipantId::new("bob")).expect("gate builds");
        gate.gate_node(
            NodeId::new(7),
            vec!["signoff".to_string(), "countersign".to_string()],
        );

        let mut runtime =
            CapabilityRuntime::with_gate(Runtime::new(machine(), Ctx, "shared".to_string()), gate);
        runtime.dispatch(RuntimeEvent::Activate(NodeId::new(7)));
        assert_eq!(runtime.model(), 1, "granted activation routed");
        assert_eq!(runtime.last_denial(), None);
    }

    #[test]
    fn ungated_nodes_pass_through_unchanged() {
        let acl = duty_separated_controller();
        let gate = CapabilityGate::new(acl, &ParticipantId::new("nobody")).expect("gate builds");

        let mut runtime =
            CapabilityRuntime::with_gate(Runtime::new(machine(), Ctx, "shared".to_string()), gate);
        // Node 7 ungated: activation flows even with a gate installed.
        runtime.dispatch(RuntimeEvent::Activate(NodeId::new(7)));
        assert_eq!(runtime.model(), 1);
        assert_eq!(runtime.last_denial(), None);
    }

    #[test]
    fn no_gate_installed_behaves_like_plain_runtime() {
        let mut runtime =
            CapabilityRuntime::new(Runtime::new(machine(), Ctx, "shared".to_string()));
        runtime.dispatch(RuntimeEvent::Activate(NodeId::new(7)));
        assert_eq!(runtime.model(), 1);
        assert_eq!(runtime.last_denial(), None);
    }

    #[test]
    fn gated_scene_marks_not_granted_nodes_disabled() {
        let mut acl = duty_separated_controller();
        let alice = acl.create_principal("alice").expect("create alice");
        acl.grant(&alice, "review").expect("grant review");
        acl.grant(&alice, "deploy").expect("grant deploy");

        let mut gate = CapabilityGate::new(acl, &ParticipantId::new("alice")).expect("gate builds");
        gate.gate_node(
            NodeId::new(7),
            vec!["review".to_string(), "deploy".to_string()],
        );
        // A second gated node whose requirements compose (σ₂·σ₂ ≠ 0).
        gate.gate_node(
            NodeId::new(8),
            vec!["signoff".to_string(), "countersign".to_string()],
        );

        let scene: Scene<Msg> = Scene::column(
            1_u64,
            vec![
                Scene::text(7_u64, "deploy now").focusable(),
                Scene::text(8_u64, "dual signoff").focusable(),
                Scene::text(9_u64, "always available").focusable(),
            ],
        );

        let projected = gate.gated_scene(&scene);
        let Scene::Column { children, .. } = &projected else {
            panic!("column preserved");
        };
        let disabled_of = |idx: usize| -> bool {
            let Scene::Text(node) = &children[idx] else {
                panic!("text preserved");
            };
            node.meta.disabled
        };
        // Impossible requirements (σ₂·σ₁₁ = 0): projects disabled.
        assert!(disabled_of(0), "impossible-gated node projects disabled");
        // Composable geometry (σ₂·σ₂ ≠ 0) but alice holds neither signoff:
        // denied by missing capability — also disabled.
        assert!(disabled_of(1), "not-held gated node projects disabled");
        // Ungated node untouched.
        assert!(!disabled_of(2), "ungated node untouched");
    }

    #[test]
    fn gate_fails_closed_on_errors() {
        // A node gated on an unregistered capability id: the controller's
        // check errors (unknown capability). The gate must fail closed —
        // suppress the activation, never open it.
        let acl = duty_separated_controller();
        let mut gate = CapabilityGate::new(acl, &ParticipantId::new("alice")).expect("gate builds");
        gate.gate_node(NodeId::new(7), vec!["nonexistent".to_string()]);

        let mut runtime =
            CapabilityRuntime::with_gate(Runtime::new(machine(), Ctx, "shared".to_string()), gate);
        runtime.dispatch(RuntimeEvent::Activate(NodeId::new(7)));
        assert_eq!(runtime.model(), 0, "error path suppresses (fail closed)");
    }
}
