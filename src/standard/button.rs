use crate::{
    Color, Effect, Key, KeyEvent, Machine, NodeId, Scene, SceneBehavior, SizeConstraint, Style,
};
use cliffy_core::{Behavior, FromGeometric, GA3, IntoGeometric, behavior};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ButtonState {
    pub activations: u64,
}

impl IntoGeometric for ButtonState {
    fn into_geometric(self) -> GA3 {
        GA3::zero()
    }
}

impl FromGeometric for ButtonState {
    fn from_geometric(_mv: &GA3) -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ButtonMsg {
    Press,
}

#[must_use]
pub fn button_key_msg(event: KeyEvent) -> Option<ButtonMsg> {
    match event.key {
        Key::Enter | Key::Char(' ') if !event.ctrl && !event.alt => Some(ButtonMsg::Press),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct ButtonContext {
    pub root_id: NodeId,
    pub label_id: NodeId,
    pub width: u16,
    pub label: String,
}

impl Default for ButtonContext {
    fn default() -> Self {
        Self {
            root_id: NodeId::new(50_000),
            label_id: NodeId::new(50_001),
            width: 12,
            label: "button".into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ButtonMachine;

impl ButtonMachine {
    #[must_use]
    pub fn key_msg(&self, event: KeyEvent) -> Option<ButtonMsg> {
        button_key_msg(event)
    }

    #[must_use]
    pub fn root_id(&self, ctx: &ButtonContext) -> NodeId {
        ctx.root_id
    }
}

impl Machine for ButtonMachine {
    type Context = ButtonContext;
    type Msg = ButtonMsg;
    type Model = ButtonState;
    type Shared = ();

    fn init(&self, _ctx: &Self::Context) -> Self::Model {
        ButtonState::default()
    }

    fn update(
        &self,
        model: &mut Self::Model,
        msg: Self::Msg,
        ctx: &Self::Context,
    ) -> Effect<Self::Msg> {
        match msg {
            ButtonMsg::Press => {
                model.activations = model.activations.saturating_add(1);
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
        let style = if model.activations > 0 {
            Style::PLAIN.fg(Color::Ansi(2)).bold()
        } else {
            Style::PLAIN.fg(Color::Ansi(6)).bold()
        };

        Scene::border(
            ctx.root_id,
            Scene::sized(
                NodeId::new(ctx.root_id.get().saturating_add(10)),
                SizeConstraint::width(ctx.width),
                Scene::text(ctx.label_id, ctx.label.clone())
                    .focusable()
                    .with_style(style)
                    .on_activate(ButtonMsg::Press),
            ),
        )
        .with_style(Style::PLAIN.fg(if model.activations > 0 {
            Color::Ansi(2)
        } else {
            Color::Ansi(8)
        }))
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
            scene_for_model.set(ButtonMachine.project_once(next_model, &(), &ctx));
        });
        scene
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderOp, Runtime, layout::Rect};

    #[test]
    fn button_key_binding_maps_enter_and_space() {
        assert_eq!(
            button_key_msg(KeyEvent {
                key: Key::Enter,
                ctrl: false,
                alt: false,
                shift: false,
            }),
            Some(ButtonMsg::Press)
        );
        assert_eq!(
            button_key_msg(KeyEvent {
                key: Key::Char(' '),
                ctrl: false,
                alt: false,
                shift: false,
            }),
            Some(ButtonMsg::Press)
        );
    }

    #[test]
    fn button_machine_tracks_activation_count() {
        let machine = ButtonMachine;
        let mut runtime = Runtime::new(
            machine,
            ButtonContext {
                label: "Run".into(),
                ..ButtonContext::default()
            },
            (),
        );

        runtime.send(ButtonMsg::Press);
        runtime.send(ButtonMsg::Press);

        assert_eq!(runtime.model(), ButtonState { activations: 2 });
    }

    #[test]
    fn button_machine_renders_focusable_label() {
        let machine = ButtonMachine;
        let runtime = Runtime::new(
            machine,
            ButtonContext {
                label: "Run".into(),
                width: 8,
                ..ButtonContext::default()
            },
            (),
        );

        assert!(
            runtime
                .render_ops(Rect::new(0, 0, 20, 3))
                .iter()
                .any(|op| matches!(op, RenderOp::DrawText { content, .. } if content == "Run"))
        );
    }
}
