use crate::{
    Anchor, FocusOrder, FocusState, Key, KeyEvent, NodeId, Padding, Scene, next_focus_in_order,
    previous_focus_in_order, trap_focus,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModalIds {
    pub root: NodeId,
    pub dialog: NodeId,
    pub backdrop: NodeId,
    pub frame: NodeId,
}

impl Default for ModalIds {
    fn default() -> Self {
        Self {
            root: NodeId::new(40_000),
            dialog: NodeId::new(40_001),
            backdrop: NodeId::new(40_002),
            frame: NodeId::new(40_003),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalMsg<Msg> {
    Inner(Msg),
    FocusPrimary,
    Dismiss,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModalFocusConfig<Msg> {
    pub scope_root: NodeId,
    pub focus_order: FocusOrder,
    pub primary_focus: NodeId,
    pub tab_forward: Msg,
    pub tab_backward: Msg,
}

#[must_use]
pub fn modal_key_msg<Msg: Clone>(
    focus: &FocusState,
    open: bool,
    event: KeyEvent,
    config: &ModalFocusConfig<Msg>,
) -> Option<ModalMsg<Msg>> {
    if !open {
        return None;
    }

    if trap_focus(focus, config.scope_root, config.primary_focus).is_some() {
        return Some(ModalMsg::FocusPrimary);
    }

    match event.key {
        Key::Escape => Some(ModalMsg::Dismiss),
        Key::Tab if event.shift => {
            let _ = previous_focus_in_order(focus, &config.focus_order);
            Some(ModalMsg::Inner(config.tab_backward.clone()))
        }
        Key::Tab => {
            let _ = next_focus_in_order(focus, &config.focus_order);
            Some(ModalMsg::Inner(config.tab_forward.clone()))
        }
        _ => None,
    }
}

#[must_use]
pub fn modal_scene<Msg: Clone>(ids: ModalIds, body: Scene<Msg>) -> Scene<ModalMsg<Msg>> {
    Scene::overlay(
        ids.root,
        vec![
            Scene::annotated(
                NodeId::new(ids.backdrop.get().saturating_add(1)),
                "modal-backdrop",
                Scene::text(ids.backdrop, " ")
                    .focusable()
                    .on_activate(ModalMsg::Dismiss),
            ),
            Scene::padding(
                ids.frame,
                Padding::all(1),
                Scene::align(ids.dialog, Anchor::center(), body.map_msg(&ModalMsg::Inner)),
            ),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FocusOrder, FocusPath, Key, activation_message};

    #[test]
    fn modal_key_msg_handles_escape_tab_and_focus_trap() {
        let ids = ModalIds::default();
        let order = FocusOrder::new(vec![ids.dialog, ids.backdrop]);
        let mut focus = FocusState::new();
        focus.set(FocusPath::from_vec(vec![ids.root, ids.dialog]));

        assert_eq!(
            modal_key_msg(
                &focus,
                true,
                KeyEvent {
                    key: Key::Escape,
                    ctrl: false,
                    alt: false,
                    shift: false,
                },
                &ModalFocusConfig {
                    scope_root: ids.root,
                    focus_order: order.clone(),
                    primary_focus: ids.dialog,
                    tab_forward: 1_u8,
                    tab_backward: 2_u8,
                },
            ),
            Some(ModalMsg::Dismiss)
        );
        assert_eq!(
            modal_key_msg(
                &focus,
                true,
                KeyEvent {
                    key: Key::Tab,
                    ctrl: false,
                    alt: false,
                    shift: false,
                },
                &ModalFocusConfig {
                    scope_root: ids.root,
                    focus_order: order.clone(),
                    primary_focus: ids.dialog,
                    tab_forward: 1_u8,
                    tab_backward: 2_u8,
                },
            ),
            Some(ModalMsg::Inner(1))
        );

        let outside_focus = FocusState::new();
        assert_eq!(
            modal_key_msg(
                &outside_focus,
                true,
                KeyEvent {
                    key: Key::Down,
                    ctrl: false,
                    alt: false,
                    shift: false,
                },
                &ModalFocusConfig {
                    scope_root: ids.root,
                    focus_order: order,
                    primary_focus: ids.dialog,
                    tab_forward: 1_u8,
                    tab_backward: 2_u8,
                },
            ),
            Some(ModalMsg::FocusPrimary)
        );
    }

    #[test]
    fn modal_scene_wraps_body_and_exposes_backdrop_dismiss() {
        let ids = ModalIds::default();
        let scene = modal_scene(ids, Scene::<()>::text(50_u64, "body"));

        assert_eq!(
            activation_message(&scene, ids.backdrop),
            Some(ModalMsg::Dismiss)
        );
    }
}
