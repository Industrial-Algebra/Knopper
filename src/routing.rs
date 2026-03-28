use crate::{FocusPath, FocusState, Interaction, NodeId, RuntimeEvent, Scene};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutedEvent<Msg> {
    Message(Msg),
    FocusChanged(FocusPath),
    Ignored,
}

#[must_use]
pub fn route_event<Msg: Clone>(
    scene: &Scene<Msg>,
    focus: &mut FocusState,
    event: RuntimeEvent,
) -> RoutedEvent<Msg> {
    match event {
        RuntimeEvent::Activate(id) => {
            activation_message(scene, id).map_or(RoutedEvent::Ignored, RoutedEvent::Message)
        }
        RuntimeEvent::Focus(id) => focus_path(scene, id).map_or(RoutedEvent::Ignored, |path| {
            focus.set(path.clone());
            RoutedEvent::FocusChanged(path)
        }),
        RuntimeEvent::Key(key) => match key.key {
            crate::Key::Enter => focus
                .current()
                .and_then(FocusPath::current)
                .and_then(|id| activation_message(scene, id))
                .map_or(RoutedEvent::Ignored, RoutedEvent::Message),
            _ => RoutedEvent::Ignored,
        },
        RuntimeEvent::Blur(_)
        | RuntimeEvent::Resize(_)
        | RuntimeEvent::Tick
        | RuntimeEvent::Sync(_) => RoutedEvent::Ignored,
    }
}

#[must_use]
pub fn focus_path<Msg>(scene: &Scene<Msg>, target: NodeId) -> Option<FocusPath> {
    let mut stack = Vec::new();
    collect_path(scene, target, &mut stack).map(FocusPath::from)
}

#[must_use]
pub fn activation_message<Msg: Clone>(scene: &Scene<Msg>, target: NodeId) -> Option<Msg> {
    match scene {
        Scene::Empty => None,
        Scene::Text(node) => {
            if node.meta.id == target {
                match &node.interaction {
                    Interaction::None => None,
                    Interaction::Activate(msg) => Some(msg.clone()),
                }
            } else {
                None
            }
        }
        Scene::Row { meta, children }
        | Scene::Column { meta, children }
        | Scene::Stack { meta, children } => {
            if meta.id == target {
                None
            } else {
                children
                    .iter()
                    .find_map(|child| activation_message(child, target))
            }
        }
        Scene::Padding { meta, child, .. }
        | Scene::Sized { meta, child, .. }
        | Scene::Viewport { meta, child }
        | Scene::Scroll { meta, child, .. }
        | Scene::Border { meta, child }
        | Scene::Annotated { meta, child, .. } => {
            if meta.id == target {
                None
            } else {
                activation_message(child, target)
            }
        }
    }
}

fn collect_path<Msg>(
    scene: &Scene<Msg>,
    target: NodeId,
    stack: &mut Vec<NodeId>,
) -> Option<Vec<NodeId>> {
    match scene {
        Scene::Empty => None,
        Scene::Text(node) => {
            stack.push(node.meta.id);
            if node.meta.id == target {
                Some(stack.clone())
            } else {
                stack.pop();
                None
            }
        }
        Scene::Row { meta, children }
        | Scene::Column { meta, children }
        | Scene::Stack { meta, children } => {
            stack.push(meta.id);
            if meta.id == target {
                return Some(stack.clone());
            }
            let found = children
                .iter()
                .find_map(|child| collect_path(child, target, stack));
            if found.is_none() {
                stack.pop();
            }
            found
        }
        Scene::Padding { meta, child, .. }
        | Scene::Sized { meta, child, .. }
        | Scene::Viewport { meta, child }
        | Scene::Scroll { meta, child, .. }
        | Scene::Border { meta, child }
        | Scene::Annotated { meta, child, .. } => {
            stack.push(meta.id);
            if meta.id == target {
                return Some(stack.clone());
            }
            let found = collect_path(child, target, stack);
            if found.is_none() {
                stack.pop();
            }
            found
        }
    }
}

impl From<Vec<NodeId>> for FocusPath {
    fn from(value: Vec<NodeId>) -> Self {
        Self::from_vec(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Key, KeyEvent};

    #[test]
    fn routes_activation_by_node_id() {
        let scene = Scene::column(
            1_u64,
            vec![
                Scene::text(2_u64, "first").on_activate("first"),
                Scene::annotated(
                    3_u64,
                    "wrap",
                    Scene::text(4_u64, "second").on_activate("second"),
                ),
            ],
        );

        let mut focus = FocusState::new();
        let routed = route_event(&scene, &mut focus, RuntimeEvent::Activate(NodeId::new(4)));

        assert_eq!(routed, RoutedEvent::Message("second"));
    }

    #[test]
    fn focus_event_updates_focus_state() {
        let scene = Scene::<()>::column(
            1_u64,
            vec![
                Scene::text(2_u64, "first"),
                Scene::text(3_u64, "second").focusable(),
            ],
        );

        let mut focus = FocusState::new();
        let routed = route_event(&scene, &mut focus, RuntimeEvent::Focus(NodeId::new(3)));

        assert_eq!(
            routed,
            RoutedEvent::FocusChanged(FocusPath::from_vec(vec![NodeId::new(1), NodeId::new(3)]))
        );
        assert_eq!(
            focus.current().and_then(FocusPath::current),
            Some(NodeId::new(3))
        );
    }

    #[test]
    fn enter_key_activates_currently_focused_node() {
        let scene = Scene::row(
            1_u64,
            vec![Scene::text(2_u64, "run").on_activate("run").focusable()],
        );
        let mut focus = FocusState::new();
        focus.set(FocusPath::from_vec(vec![NodeId::new(1), NodeId::new(2)]));

        let routed = route_event(
            &scene,
            &mut focus,
            RuntimeEvent::Key(KeyEvent {
                key: Key::Enter,
                ctrl: false,
                alt: false,
                shift: false,
            }),
        );

        assert_eq!(routed, RoutedEvent::Message("run"));
    }
}
