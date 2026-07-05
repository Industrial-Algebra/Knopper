// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use crate::{Scene, id::NodeId, scene::FocusScopePolicy};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FocusOrder(Vec<NodeId>);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FocusPath(Vec<NodeId>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusNavigation {
    pub order: FocusOrder,
    pub policy: FocusScopePolicy,
}

impl FocusOrder {
    #[must_use]
    pub fn new(ids: impl Into<Vec<NodeId>>) -> Self {
        Self(ids.into())
    }

    #[must_use]
    pub fn collect_from_scene<Msg>(scene: &Scene<Msg>) -> Self {
        let mut ids = Vec::new();
        collect_focusable(scene, &mut ids);
        Self(ids)
    }

    #[must_use]
    pub fn collect_from_scope<Msg>(scene: &Scene<Msg>, scope: &str) -> Option<Self> {
        let mut ids = Vec::new();
        collect_focusable_in_scope(scene, scope, &mut ids).then_some(Self(ids))
    }

    #[must_use]
    pub fn collect_from_nearest_scope<Msg>(scene: &Scene<Msg>, target: NodeId) -> Option<Self> {
        let scope = nearest_scope_name(scene, target)?;
        Self::collect_from_scope(scene, &scope)
    }

    #[must_use]
    pub fn collect_from_focus_path<Msg>(scene: &Scene<Msg>, path: &FocusPath) -> Option<Self> {
        path.current()
            .and_then(|current| Self::collect_from_nearest_scope(scene, current))
    }

    #[must_use]
    pub fn collect_for_focus<Msg>(scene: &Scene<Msg>, focus: &FocusState) -> Self {
        FocusNavigation::for_focus(scene, focus).order
    }

    #[must_use]
    pub fn as_slice(&self) -> &[NodeId] {
        &self.0
    }

    #[must_use]
    pub fn first(&self) -> Option<NodeId> {
        self.0.first().copied()
    }

    #[must_use]
    pub fn next(&self, current: Option<NodeId>) -> Option<NodeId> {
        match current.and_then(|id| self.0.iter().position(|candidate| *candidate == id)) {
            Some(index) => self.0.get((index + 1) % self.0.len()).copied(),
            None => self.first(),
        }
    }

    #[must_use]
    pub fn previous(&self, current: Option<NodeId>) -> Option<NodeId> {
        match current.and_then(|id| self.0.iter().position(|candidate| *candidate == id)) {
            Some(0) => self.0.last().copied(),
            Some(index) => self.0.get(index - 1).copied(),
            None => self.first(),
        }
    }

    #[must_use]
    pub fn next_non_wrapping(&self, current: Option<NodeId>) -> Option<NodeId> {
        match current.and_then(|id| self.0.iter().position(|candidate| *candidate == id)) {
            Some(index) => self.0.get(index + 1).copied(),
            None => self.first(),
        }
    }

    #[must_use]
    pub fn previous_non_wrapping(&self, current: Option<NodeId>) -> Option<NodeId> {
        match current.and_then(|id| self.0.iter().position(|candidate| *candidate == id)) {
            Some(index) if index > 0 => self.0.get(index - 1).copied(),
            Some(_) => None,
            None => self.first(),
        }
    }
}

impl FocusNavigation {
    #[must_use]
    pub fn for_focus<Msg>(scene: &Scene<Msg>, focus: &FocusState) -> Self {
        let current = focus.current().and_then(FocusPath::current);
        current
            .and_then(|id| nearest_scope_descriptor(scene, id))
            .and_then(|descriptor| {
                FocusOrder::collect_from_scope(scene, &descriptor.name).map(|order| Self {
                    order,
                    policy: descriptor.policy,
                })
            })
            .unwrap_or_else(|| Self {
                order: FocusOrder::collect_from_scene(scene),
                policy: FocusScopePolicy::Wrap,
            })
    }

    #[must_use]
    pub fn advance(
        &self,
        scene_order: &FocusOrder,
        current: Option<NodeId>,
        backward: bool,
    ) -> Option<NodeId> {
        match (self.policy, backward) {
            // Wrap cycles within the scope and never escapes.
            (FocusScopePolicy::Wrap, false) => self.order.next(current),
            (FocusScopePolicy::Wrap, true) => self.order.previous(current),
            // Trap clamps at scope boundaries: focus may move inside the
            // scope but can never leave it. This is what distinguishes Trap
            // from Wrap and lets modal dialogs rely on the declarative
            // policy instead of imperative trap_focus helpers.
            (FocusScopePolicy::Trap, false) => self.order.next_non_wrapping(current),
            (FocusScopePolicy::Trap, true) => self.order.previous_non_wrapping(current),
            (FocusScopePolicy::Local, false) => self.order.next_non_wrapping(current),
            (FocusScopePolicy::Local, true) => self.order.previous_non_wrapping(current),
            (FocusScopePolicy::Passthrough, false) => self
                .order
                .next_non_wrapping(current)
                .or_else(|| scene_order.next(current)),
            (FocusScopePolicy::Passthrough, true) => self
                .order
                .previous_non_wrapping(current)
                .or_else(|| scene_order.previous(current)),
        }
    }
}

fn collect_focusable<Msg>(scene: &Scene<Msg>, ids: &mut Vec<NodeId>) {
    match scene {
        Scene::Empty => {}
        Scene::Text(node) => {
            if node.meta.focusable && !node.meta.disabled {
                ids.push(node.meta.id);
            }
        }
        Scene::Row { meta, children }
        | Scene::Column { meta, children }
        | Scene::Stack { meta, children } => {
            if meta.focusable && !meta.disabled {
                ids.push(meta.id);
            }
            for child in children {
                collect_focusable(child, ids);
            }
        }
        Scene::FocusScope { meta, child, .. }
        | Scene::Align { meta, child, .. }
        | Scene::Padding { meta, child, .. }
        | Scene::Sized { meta, child, .. }
        | Scene::Viewport { meta, child, .. }
        | Scene::Scroll { meta, child, .. }
        | Scene::Border { meta, child, .. }
        | Scene::Annotated { meta, child, .. } => {
            if meta.focusable && !meta.disabled {
                ids.push(meta.id);
            }
            collect_focusable(child, ids);
        }
    }
}

fn collect_focusable_in_scope<Msg>(scene: &Scene<Msg>, scope: &str, ids: &mut Vec<NodeId>) -> bool {
    match scene {
        Scene::Empty | Scene::Text(_) => false,
        Scene::Row { children, .. }
        | Scene::Column { children, .. }
        | Scene::Stack { children, .. } => children
            .iter()
            .any(|child| collect_focusable_in_scope(child, scope, ids)),
        Scene::FocusScope { name, child, .. } => {
            if name == scope {
                collect_focusable(child, ids);
                true
            } else {
                collect_focusable_in_scope(child, scope, ids)
            }
        }
        Scene::Align { child, .. }
        | Scene::Padding { child, .. }
        | Scene::Sized { child, .. }
        | Scene::Viewport { child, .. }
        | Scene::Scroll { child, .. }
        | Scene::Border { child, .. }
        | Scene::Annotated { child, .. } => collect_focusable_in_scope(child, scope, ids),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FocusScopeDescriptor {
    name: String,
    policy: FocusScopePolicy,
}

fn nearest_scope_name<Msg>(scene: &Scene<Msg>, target: NodeId) -> Option<String> {
    nearest_scope_descriptor(scene, target).map(|descriptor| descriptor.name)
}

fn nearest_scope_descriptor<Msg>(
    scene: &Scene<Msg>,
    target: NodeId,
) -> Option<FocusScopeDescriptor> {
    nearest_scope_descriptor_within(scene, target, None)
}

fn nearest_scope_descriptor_within<Msg>(
    scene: &Scene<Msg>,
    target: NodeId,
    current_scope: Option<&FocusScopeDescriptor>,
) -> Option<FocusScopeDescriptor> {
    match scene {
        Scene::Empty => None,
        Scene::Text(node) => (node.meta.id == target)
            .then(|| current_scope.cloned())
            .flatten(),
        Scene::Row { meta, children }
        | Scene::Column { meta, children }
        | Scene::Stack { meta, children } => {
            if meta.id == target {
                current_scope.cloned()
            } else {
                children
                    .iter()
                    .find_map(|child| nearest_scope_descriptor_within(child, target, current_scope))
            }
        }
        Scene::FocusScope {
            meta,
            name,
            policy,
            child,
        } => {
            if meta.id == target {
                current_scope.cloned()
            } else {
                let scope = FocusScopeDescriptor {
                    name: name.clone(),
                    policy: *policy,
                };
                nearest_scope_descriptor_within(child, target, Some(&scope))
                    .or_else(|| nearest_scope_descriptor_within(child, target, current_scope))
            }
        }
        Scene::Align { meta, child, .. }
        | Scene::Padding { meta, child, .. }
        | Scene::Sized { meta, child, .. }
        | Scene::Viewport { meta, child, .. }
        | Scene::Scroll { meta, child, .. }
        | Scene::Border { meta, child, .. }
        | Scene::Annotated { meta, child, .. } => {
            if meta.id == target {
                current_scope.cloned()
            } else {
                nearest_scope_descriptor_within(child, target, current_scope)
            }
        }
    }
}

impl FocusPath {
    #[must_use]
    pub fn root(root: NodeId) -> Self {
        Self(vec![root])
    }

    #[must_use]
    pub fn child(mut self, id: NodeId) -> Self {
        self.0.push(id);
        self
    }

    #[must_use]
    pub fn from_vec(path: Vec<NodeId>) -> Self {
        Self(path)
    }

    #[must_use]
    pub fn current(&self) -> Option<NodeId> {
        self.0.last().copied()
    }

    #[must_use]
    pub fn as_slice(&self) -> &[NodeId] {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FocusState {
    current: Option<FocusPath>,
}

impl FocusState {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, path: FocusPath) {
        self.current = Some(path);
    }

    #[must_use]
    pub fn current(&self) -> Option<&FocusPath> {
        self.current.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_path_tracks_current_node() {
        let path = FocusPath::root(NodeId::new(1)).child(NodeId::new(2));
        assert_eq!(path.current(), Some(NodeId::new(2)));
        assert_eq!(path.as_slice(), &[NodeId::new(1), NodeId::new(2)]);
    }

    #[test]
    fn focus_order_cycles_forward_and_backward() {
        let order = FocusOrder::new(vec![NodeId::new(10), NodeId::new(20), NodeId::new(30)]);

        assert_eq!(order.first(), Some(NodeId::new(10)));
        assert_eq!(order.next(Some(NodeId::new(10))), Some(NodeId::new(20)));
        assert_eq!(order.next(Some(NodeId::new(30))), Some(NodeId::new(10)));
        assert_eq!(order.previous(Some(NodeId::new(10))), Some(NodeId::new(30)));
        assert_eq!(order.previous(Some(NodeId::new(20))), Some(NodeId::new(10)));
        assert_eq!(order.next(None), Some(NodeId::new(10)));
        assert_eq!(order.next_non_wrapping(Some(NodeId::new(30))), None);
        assert_eq!(order.previous_non_wrapping(Some(NodeId::new(10))), None);
    }

    #[test]
    fn collects_focus_order_from_scene_tree_in_traversal_order() {
        let scene = Scene::<()>::column(
            1_u64,
            vec![
                Scene::text(2_u64, "a").focusable(),
                Scene::align(
                    3_u64,
                    crate::Anchor::center(),
                    Scene::row(
                        4_u64,
                        vec![
                            Scene::text(5_u64, "b").focusable(),
                            Scene::text(6_u64, "c"),
                            Scene::text(7_u64, "d").focusable(),
                        ],
                    ),
                ),
                Scene::border(8_u64, Scene::text(9_u64, "e").focusable()),
            ],
        );

        let order = FocusOrder::collect_from_scene(&scene);
        assert_eq!(
            order.as_slice(),
            &[
                NodeId::new(2),
                NodeId::new(5),
                NodeId::new(7),
                NodeId::new(9)
            ]
        );
    }

    #[test]
    fn collects_focus_order_from_named_scope() {
        let scene = Scene::<()>::column(
            1_u64,
            vec![
                Scene::focus_scope(
                    2_u64,
                    "alpha",
                    Scene::row(
                        3_u64,
                        vec![
                            Scene::text(4_u64, "a").focusable(),
                            Scene::text(5_u64, "b").focusable(),
                        ],
                    ),
                ),
                Scene::focus_scope(
                    6_u64,
                    "beta",
                    Scene::column(
                        7_u64,
                        vec![
                            Scene::text(8_u64, "c").focusable(),
                            Scene::text(9_u64, "d").focusable(),
                        ],
                    ),
                ),
            ],
        );

        assert_eq!(
            FocusOrder::collect_from_scope(&scene, "beta")
                .expect("scope should exist")
                .as_slice(),
            &[NodeId::new(8), NodeId::new(9)]
        );
        assert!(FocusOrder::collect_from_scope(&scene, "missing").is_none());
    }

    #[test]
    fn derives_focus_order_from_nearest_scope_and_focus_path() {
        let scene = Scene::<()>::column(
            1_u64,
            vec![
                Scene::focus_scope_with_policy(
                    2_u64,
                    "alpha",
                    FocusScopePolicy::Passthrough,
                    Scene::row(
                        3_u64,
                        vec![
                            Scene::text(4_u64, "a").focusable(),
                            Scene::focus_scope_with_policy(
                                5_u64,
                                "alpha-inner",
                                FocusScopePolicy::Wrap,
                                Scene::column(
                                    6_u64,
                                    vec![
                                        Scene::text(7_u64, "b").focusable(),
                                        Scene::text(8_u64, "c").focusable(),
                                    ],
                                ),
                            ),
                        ],
                    ),
                ),
                Scene::text(9_u64, "d").focusable(),
            ],
        );

        assert_eq!(
            FocusOrder::collect_from_nearest_scope(&scene, NodeId::new(7))
                .expect("nested scope should exist")
                .as_slice(),
            &[NodeId::new(7), NodeId::new(8)]
        );

        let path = FocusPath::from_vec(vec![
            NodeId::new(1),
            NodeId::new(2),
            NodeId::new(5),
            NodeId::new(7),
        ]);
        assert_eq!(
            FocusOrder::collect_from_focus_path(&scene, &path)
                .expect("focus path scope should exist")
                .as_slice(),
            &[NodeId::new(7), NodeId::new(8)]
        );

        let mut focus = FocusState::new();
        focus.set(path);
        assert_eq!(
            FocusOrder::collect_for_focus(&scene, &focus).as_slice(),
            &[NodeId::new(7), NodeId::new(8)]
        );
    }

    #[test]
    fn derives_navigation_policy_from_nearest_scope() {
        let scene = Scene::<()>::column(
            1_u64,
            vec![
                Scene::text(2_u64, "outside").focusable(),
                Scene::focus_scope_with_policy(
                    3_u64,
                    "local",
                    FocusScopePolicy::Local,
                    Scene::column(
                        4_u64,
                        vec![
                            Scene::text(5_u64, "a").focusable(),
                            Scene::text(6_u64, "b").focusable(),
                        ],
                    ),
                ),
            ],
        );
        let mut focus = FocusState::new();
        focus.set(FocusPath::from_vec(vec![
            NodeId::new(1),
            NodeId::new(3),
            NodeId::new(4),
            NodeId::new(6),
        ]));

        let navigation = FocusNavigation::for_focus(&scene, &focus);
        assert_eq!(navigation.policy, FocusScopePolicy::Local);
        assert_eq!(
            navigation.order.as_slice(),
            &[NodeId::new(5), NodeId::new(6)]
        );
        assert_eq!(
            navigation.advance(
                &FocusOrder::collect_from_scene(&scene),
                Some(NodeId::new(6)),
                false
            ),
            None
        );
    }

    #[test]
    fn trap_scope_clamps_at_boundaries_instead_of_wrapping() {
        // Trap semantics: focus must NEVER leave the scope.
        // Advancing forward at the last element returns None (clamps),
        // it does NOT wrap back to the first element. Likewise backward
        // at the first element returns None.
        // This is what distinguishes Trap from Wrap, and it is what lets
        // modal dialogs rely on the declarative scope policy instead of
        // imperative trap_focus helpers.
        let scene = Scene::<()>::focus_scope_with_policy(
            1_u64,
            "trap",
            FocusScopePolicy::Trap,
            Scene::column(
                2_u64,
                vec![
                    Scene::text(3_u64, "a").focusable(),
                    Scene::text(4_u64, "b").focusable(),
                    Scene::text(5_u64, "c").focusable(),
                ],
            ),
        );
        let mut focus = FocusState::new();
        focus.set(FocusPath::from_vec(vec![
            NodeId::new(1),
            NodeId::new(2),
            NodeId::new(5),
        ]));

        let navigation = FocusNavigation::for_focus(&scene, &focus);
        assert_eq!(navigation.policy, FocusScopePolicy::Trap);
        assert_eq!(
            navigation.order.as_slice(),
            &[NodeId::new(3), NodeId::new(4), NodeId::new(5)]
        );

        // Forward at the last element: clamp (None), do NOT wrap to first.
        assert_eq!(
            navigation.advance(
                &FocusOrder::collect_from_scene(&scene),
                Some(NodeId::new(5)),
                false
            ),
            None
        );

        // Backward at the first element: clamp (None).
        let mut first_focus = FocusState::new();
        first_focus.set(FocusPath::from_vec(vec![
            NodeId::new(1),
            NodeId::new(2),
            NodeId::new(3),
        ]));
        let first_nav = FocusNavigation::for_focus(&scene, &first_focus);
        assert_eq!(
            first_nav.advance(
                &FocusOrder::collect_from_scene(&scene),
                Some(NodeId::new(3)),
                true
            ),
            None
        );

        // Interior traversal still works.
        assert_eq!(
            navigation.advance(
                &FocusOrder::collect_from_scene(&scene),
                Some(NodeId::new(4)),
                false
            ),
            Some(NodeId::new(5))
        );
    }

    #[test]
    fn disabled_nodes_are_skipped_by_focus_collection() {
        use crate::FocusOrder;
        let scene = Scene::<()>::column(
            1_u64,
            vec![
                Scene::text(2_u64, "a").focusable(),
                Scene::text(3_u64, "b").focusable().disabled(),
                Scene::text(4_u64, "c").focusable(),
            ],
        );

        // Focus order skips the disabled node.
        let order = FocusOrder::collect_from_scene(&scene);
        assert_eq!(order.as_slice(), &[NodeId::new(2), NodeId::new(4)]);
    }

    #[test]
    fn passthrough_scope_falls_back_to_scene_order_at_boundaries() {
        let scene = Scene::<()>::column(
            1_u64,
            vec![
                Scene::text(2_u64, "outside-a").focusable(),
                Scene::focus_scope_with_policy(
                    3_u64,
                    "pass",
                    FocusScopePolicy::Passthrough,
                    Scene::column(
                        4_u64,
                        vec![
                            Scene::text(5_u64, "inside-a").focusable(),
                            Scene::text(6_u64, "inside-b").focusable(),
                        ],
                    ),
                ),
                Scene::text(7_u64, "outside-b").focusable(),
            ],
        );
        let mut focus = FocusState::new();
        focus.set(FocusPath::from_vec(vec![
            NodeId::new(1),
            NodeId::new(3),
            NodeId::new(4),
            NodeId::new(6),
        ]));

        let navigation = FocusNavigation::for_focus(&scene, &focus);
        assert_eq!(navigation.policy, FocusScopePolicy::Passthrough);
        assert_eq!(
            navigation.advance(
                &FocusOrder::collect_from_scene(&scene),
                Some(NodeId::new(6)),
                false
            ),
            Some(NodeId::new(7))
        );
    }
}
