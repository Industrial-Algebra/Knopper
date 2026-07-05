// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use crate::{Anchor, FocusOrder, FocusState, Key, KeyEvent, NodeId, Padding, Scene, trap_focus};

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

/**
 * Focus configuration for a declarative modal.
 *
 * Tab and Shift-Tab are handled by the runtime's declarative
 * `FocusNavigation` against the modal's `FocusScopePolicy::Trap` scope —
 * they do not flow through `modal_key_msg`. This struct carries only what
 * the imperative modal keybindings need: Escape dismissal and the
 * focus-refocus safety net that pulls focus back into the modal if it
 * somehow escaped the trap scope.
 */
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModalFocusConfig {
    /// Node id of the modal root. Used by the focus-refocus safety net.
    pub scope_root: NodeId,
    /// Focusable node inside the modal that should receive focus when the
    /// safety net fires.
    pub primary_focus: NodeId,
    /// Focus order of the modal scope. Retained for introspection and for
    /// machines that still want to compute it, but no longer used for
    /// imperative Tab cycling.
    pub focus_order: FocusOrder,
}

/// Outcome of an imperative modal keybinding.
///
/// `modal_key_msg` only ever produces these two variants — Tab/Shift-Tab
/// are handled declaratively by the runtime, so there is no `Inner`
/// variant here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalKeyAction {
    /// The modal should close.
    Dismiss,
    /// Focus escaped the trap scope and should be pulled back to the
    /// modal's primary focusable node.
    FocusPrimary,
}

/// Imperative modal keybindings.
///
/// Returns `Some(ModalKeyAction)` only for modal-specific concerns:
/// - `Escape` → `Dismiss`
/// - any key while focus is outside the trap scope → `FocusPrimary`
///   (defensive refocus; with correct `FocusScopePolicy::Trap` semantics
///   and an open-action that sets focus into the scope, this is a safety
///   net rather than the primary trapping mechanism).
///
/// Returns `None` for everything else — including `Tab` and `Shift-Tab`,
/// which are handled declaratively by the runtime against the modal's
/// `FocusScopePolicy::Trap` scope.
#[must_use]
pub fn modal_key_msg(
    focus: &FocusState,
    open: bool,
    event: KeyEvent,
    config: &ModalFocusConfig,
) -> Option<ModalKeyAction> {
    if !open {
        return None;
    }

    if trap_focus(focus, config.scope_root, config.primary_focus).is_some() {
        return Some(ModalKeyAction::FocusPrimary);
    }

    if event.key == Key::Escape {
        return Some(ModalKeyAction::Dismiss);
    }

    None
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
    fn modal_key_msg_handles_escape_defocus_safety_net_and_defers_tab() {
        // After migration: modal_key_msg only handles Escape and the
        // focus-refocus safety net. Tab/Shift-Tab return None and are
        // handled declaratively by the runtime against the Trap scope.
        let ids = ModalIds::default();
        let order = FocusOrder::new(vec![ids.dialog, ids.backdrop]);
        let config = ModalFocusConfig {
            scope_root: ids.root,
            focus_order: order,
            primary_focus: ids.dialog,
        };

        // Escape dismisses.
        let mut inside_focus = FocusState::new();
        inside_focus.set(FocusPath::from_vec(vec![ids.root, ids.dialog]));
        assert_eq!(
            modal_key_msg(
                &inside_focus,
                true,
                KeyEvent {
                    key: Key::Escape,
                    ctrl: false,
                    alt: false,
                    shift: false,
                },
                &config,
            ),
            Some(ModalKeyAction::Dismiss)
        );

        // Tab is deferred to the runtime (None).
        assert_eq!(
            modal_key_msg(
                &inside_focus,
                true,
                KeyEvent {
                    key: Key::Tab,
                    ctrl: false,
                    alt: false,
                    shift: false,
                },
                &config,
            ),
            None
        );
        assert_eq!(
            modal_key_msg(
                &inside_focus,
                true,
                KeyEvent {
                    key: Key::Tab,
                    ctrl: false,
                    alt: false,
                    shift: true,
                },
                &config,
            ),
            None
        );

        // Focus outside the trap scope → any key refocuses (safety net).
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
                &config,
            ),
            Some(ModalKeyAction::FocusPrimary)
        );

        // Closed modal produces nothing.
        assert_eq!(
            modal_key_msg(
                &inside_focus,
                false,
                KeyEvent {
                    key: Key::Escape,
                    ctrl: false,
                    alt: false,
                    shift: false,
                },
                &config,
            ),
            None
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
