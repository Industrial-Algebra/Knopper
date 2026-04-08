use crate::{
    Color, Effect, Key, KeyEvent, Machine, NodeId, Role, Scene, SceneBehavior, SizeConstraint,
    Style,
};
use cliffy_core::{Behavior, FromGeometric, GA3, IntoGeometric, behavior};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ToggleState {
    pub checked: bool,
}

impl IntoGeometric for ToggleState {
    fn into_geometric(self) -> GA3 {
        GA3::zero()
    }
}

impl FromGeometric for ToggleState {
    fn from_geometric(_mv: &GA3) -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToggleMsg {
    Toggle,
    Set(bool),
}

#[must_use]
pub fn toggle_key_msg(event: KeyEvent) -> Option<ToggleMsg> {
    match event.key {
        Key::Enter | Key::Char(' ') if !event.ctrl && !event.alt => Some(ToggleMsg::Toggle),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct ToggleContext {
    pub root_id: NodeId,
    pub box_id: NodeId,
    pub label_id: NodeId,
    pub width: u16,
    pub label: String,
}

impl Default for ToggleContext {
    fn default() -> Self {
        Self {
            root_id: NodeId::new(60_000),
            box_id: NodeId::new(60_001),
            label_id: NodeId::new(60_002),
            width: 18,
            label: "toggle".into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ToggleMachine;

impl ToggleMachine {
    #[must_use]
    pub fn key_msg(&self, event: KeyEvent) -> Option<ToggleMsg> {
        toggle_key_msg(event)
    }

    #[must_use]
    pub fn root_id(&self, ctx: &ToggleContext) -> NodeId {
        ctx.root_id
    }
}

impl Machine for ToggleMachine {
    type Context = ToggleContext;
    type Msg = ToggleMsg;
    type Model = ToggleState;
    type Shared = ();

    fn init(&self, _ctx: &Self::Context) -> Self::Model {
        ToggleState::default()
    }

    fn update(
        &self,
        model: &mut Self::Model,
        msg: Self::Msg,
        ctx: &Self::Context,
    ) -> Effect<Self::Msg> {
        match msg {
            ToggleMsg::Toggle => {
                model.checked = !model.checked;
                Effect::RequestFocus(ctx.root_id)
            }
            ToggleMsg::Set(value) => {
                model.checked = value;
                Effect::RequestFocus(ctx.root_id)
            }
        }
    }

    fn project_once(
        &self,
        model: &Self::Model,
        _shared: &Self::Shared,
        ctx: &Self::Context,
    ) -> Scene<Self::Msg> {
        let marker = if model.checked { "[x]" } else { "[ ]" };
        let style = if model.checked {
            Style::PLAIN.fg(Color::Ansi(2)).bold()
        } else {
            Style::PLAIN.fg(Color::Ansi(6)).bold()
        };

        Scene::border(
            ctx.root_id,
            Scene::sized(
                NodeId::new(ctx.root_id.get().saturating_add(10)),
                SizeConstraint::width(ctx.width),
                Scene::row(
                    NodeId::new(ctx.root_id.get().saturating_add(11)),
                    vec![
                        Scene::text(ctx.box_id, marker)
                            .with_role(Role::ListItem)
                            .with_style(style.bg(Color::Ansi(0))),
                        Scene::text(ctx.label_id, ctx.label.clone())
                            .focusable()
                            .with_style(style.bg(Color::Ansi(0)))
                            .on_activate(ToggleMsg::Toggle),
                    ],
                ),
            ),
        )
        .with_style(
            Style::PLAIN
                .fg(if model.checked {
                    Color::Ansi(2)
                } else {
                    Color::Ansi(8)
                })
                .bg(Color::Ansi(0)),
        )
    }

    fn project(
        &self,
        model: Behavior<Self::Model>,
        _shared: Behavior<Self::Shared>,
        ctx: &Self::Context,
    ) -> SceneBehavior<Self::Msg> {
        let scene = behavior(self.project_once(&model.sample(), &(), ctx));
        let scene_for_model = scene.clone();
        let ctx = ctx.clone();
        model.subscribe(move |next_model| {
            scene_for_model.set(ToggleMachine.project_once(next_model, &(), &ctx));
        });
        scene
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderOp, Runtime, layout::Rect};

    #[test]
    fn toggle_key_binding_maps_enter_and_space() {
        assert_eq!(
            toggle_key_msg(KeyEvent {
                key: Key::Enter,
                ctrl: false,
                alt: false,
                shift: false,
            }),
            Some(ToggleMsg::Toggle)
        );
        assert_eq!(
            toggle_key_msg(KeyEvent {
                key: Key::Char(' '),
                ctrl: false,
                alt: false,
                shift: false,
            }),
            Some(ToggleMsg::Toggle)
        );
    }

    #[test]
    fn toggle_machine_toggles_checked_state() {
        let machine = ToggleMachine;
        let mut runtime = Runtime::new(
            machine,
            ToggleContext {
                label: "Enabled".into(),
                ..ToggleContext::default()
            },
            (),
        );

        runtime.send(ToggleMsg::Toggle);
        assert_eq!(runtime.model(), ToggleState { checked: true });

        runtime.send(ToggleMsg::Set(false));
        assert_eq!(runtime.model(), ToggleState { checked: false });
    }

    #[test]
    fn toggle_machine_renders_checkbox_marker_and_label() {
        let machine = ToggleMachine;
        let runtime = Runtime::new(
            machine,
            ToggleContext {
                label: "Enabled".into(),
                ..ToggleContext::default()
            },
            (),
        );

        let ops = runtime.render_ops(Rect::new(0, 0, 24, 3));
        assert!(
            ops.iter()
                .any(|op| matches!(op, RenderOp::DrawText { content, .. } if content == "[ ]"))
        );
        assert!(
            ops.iter()
                .any(|op| matches!(op, RenderOp::DrawText { content, .. } if content == "Enabled"))
        );
    }
}
