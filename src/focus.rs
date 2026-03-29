use crate::{Scene, id::NodeId};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FocusOrder(Vec<NodeId>);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FocusPath(Vec<NodeId>);

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
        focus
            .current()
            .and_then(|path| Self::collect_from_focus_path(scene, path))
            .unwrap_or_else(|| Self::collect_from_scene(scene))
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
}

fn collect_focusable<Msg>(scene: &Scene<Msg>, ids: &mut Vec<NodeId>) {
    match scene {
        Scene::Empty => {}
        Scene::Text(node) => {
            if node.meta.focusable {
                ids.push(node.meta.id);
            }
        }
        Scene::Row { meta, children }
        | Scene::Column { meta, children }
        | Scene::Stack { meta, children } => {
            if meta.focusable {
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
            if meta.focusable {
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

fn nearest_scope_name<Msg>(scene: &Scene<Msg>, target: NodeId) -> Option<String> {
    nearest_scope_name_within(scene, target, None)
}

fn nearest_scope_name_within<Msg>(
    scene: &Scene<Msg>,
    target: NodeId,
    current_scope: Option<&str>,
) -> Option<String> {
    match scene {
        Scene::Empty => None,
        Scene::Text(node) => (node.meta.id == target)
            .then(|| current_scope.map(str::to_owned))
            .flatten(),
        Scene::Row { meta, children }
        | Scene::Column { meta, children }
        | Scene::Stack { meta, children } => {
            if meta.id == target {
                current_scope.map(str::to_owned)
            } else {
                children
                    .iter()
                    .find_map(|child| nearest_scope_name_within(child, target, current_scope))
            }
        }
        Scene::FocusScope { meta, name, child } => {
            if meta.id == target {
                current_scope.map(str::to_owned)
            } else {
                nearest_scope_name_within(child, target, Some(name))
                    .or_else(|| nearest_scope_name_within(child, target, current_scope))
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
                current_scope.map(str::to_owned)
            } else {
                nearest_scope_name_within(child, target, current_scope)
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
                Scene::focus_scope(
                    2_u64,
                    "alpha",
                    Scene::row(
                        3_u64,
                        vec![
                            Scene::text(4_u64, "a").focusable(),
                            Scene::focus_scope(
                                5_u64,
                                "alpha-inner",
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
}
