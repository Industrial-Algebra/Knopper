// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use crate::{
    Color, Effect, Key, KeyEvent, LayoutNode, Machine, NodeId, Role, Scene, SceneBehavior,
    SizeConstraint, Style, find_node,
};
use cliffy_core::{Behavior, FromGeometric, GA3, IntoGeometric, behavior};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InputState {
    pub value: String,
    pub cursor: usize,
    pub committed: Option<String>,
}

impl IntoGeometric for InputState {
    fn into_geometric(self) -> GA3 {
        GA3::zero()
    }
}

impl FromGeometric for InputState {
    fn from_geometric(_mv: &GA3) -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMsg {
    Insert(char),
    Backspace,
    MoveLeft,
    MoveRight,
    Commit,
}

#[must_use]
pub fn input_key_msg(event: KeyEvent) -> Option<InputMsg> {
    match event.key {
        Key::Char(ch) if !event.ctrl && !event.alt => Some(InputMsg::Insert(ch)),
        Key::Backspace => Some(InputMsg::Backspace),
        Key::Left => Some(InputMsg::MoveLeft),
        Key::Right => Some(InputMsg::MoveRight),
        Key::Enter => Some(InputMsg::Commit),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct InputContext {
    pub root_id: NodeId,
    pub field_id: NodeId,
    pub width: u16,
    pub placeholder: String,
}

impl Default for InputContext {
    fn default() -> Self {
        Self {
            root_id: NodeId::new(20_000),
            field_id: NodeId::new(20_001),
            width: 24,
            placeholder: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct InputMachine;

impl InputMachine {
    #[must_use]
    pub fn key_msg(&self, event: KeyEvent) -> Option<InputMsg> {
        input_key_msg(event)
    }

    #[must_use]
    pub fn root_id(&self, ctx: &InputContext) -> NodeId {
        ctx.root_id
    }
}

impl Machine for InputMachine {
    type Context = InputContext;
    type Msg = InputMsg;
    type Model = InputState;
    type Shared = ();

    fn init(&self, _ctx: &Self::Context) -> Self::Model {
        InputState::default()
    }

    fn update(
        &self,
        model: &mut Self::Model,
        msg: Self::Msg,
        _ctx: &Self::Context,
    ) -> Effect<Self::Msg> {
        match msg {
            InputMsg::Insert(ch) => {
                let byte_idx = char_to_byte_index(&model.value, model.cursor);
                model.value.insert(byte_idx, ch);
                model.cursor = model.cursor.saturating_add(1);
            }
            InputMsg::Backspace => {
                if model.cursor > 0 {
                    let remove_at = model.cursor - 1;
                    let start = char_to_byte_index(&model.value, remove_at);
                    let end = char_to_byte_index(&model.value, model.cursor);
                    model.value.replace_range(start..end, "");
                    model.cursor = remove_at;
                }
            }
            InputMsg::MoveLeft => {
                model.cursor = model.cursor.saturating_sub(1);
            }
            InputMsg::MoveRight => {
                model.cursor = (model.cursor + 1).min(model.value.chars().count());
            }
            InputMsg::Commit => {
                model.committed = Some(model.value.clone());
            }
        }
        Effect::None
    }

    fn project_once(
        &self,
        model: &Self::Model,
        _shared: &Self::Shared,
        ctx: &Self::Context,
    ) -> Scene<Self::Msg> {
        let display = if model.value.is_empty() {
            ctx.placeholder.clone()
        } else {
            model.value.clone()
        };

        Scene::border(
            ctx.root_id,
            Scene::sized(
                NodeId::new(ctx.root_id.get().saturating_add(10)),
                SizeConstraint::width(ctx.width),
                Scene::text(ctx.field_id, display)
                    .with_role(Role::Editor)
                    .focusable()
                    .with_style(if model.value.is_empty() {
                        Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0))
                    } else {
                        Style::PLAIN.fg(Color::Ansi(6)).bg(Color::Ansi(0)).bold()
                    }),
            ),
        )
        .with_style(Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0)))
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
            scene_for_model.set(InputMachine.project_once(next_model, &(), &ctx));
        });
        scene
    }

    fn cursor_position(
        &self,
        model: &Self::Model,
        _shared: &Self::Shared,
        ctx: &Self::Context,
        layout: &LayoutNode,
    ) -> Option<(u16, u16)> {
        let field = find_node(layout, ctx.field_id)?;
        let cursor_x = field.rect.x.saturating_add(
            u16::try_from(model.cursor)
                .unwrap_or(u16::MAX)
                .min(field.rect.width.saturating_sub(1)),
        );
        Some((cursor_x, field.rect.y))
    }
}

fn char_to_byte_index(value: &str, char_index: usize) -> usize {
    value
        .char_indices()
        .nth(char_index)
        .map_or(value.len(), |(idx, _)| idx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderOp, Runtime, layout::Rect};

    #[test]
    fn input_key_binding_maps_editor_keys() {
        assert_eq!(
            input_key_msg(KeyEvent {
                key: Key::Char('a'),
                ctrl: false,
                alt: false,
                shift: false,
            }),
            Some(InputMsg::Insert('a'))
        );
        assert_eq!(
            input_key_msg(KeyEvent {
                key: Key::Left,
                ctrl: false,
                alt: false,
                shift: false,
            }),
            Some(InputMsg::MoveLeft)
        );
        assert_eq!(
            input_key_msg(KeyEvent {
                key: Key::Enter,
                ctrl: false,
                alt: false,
                shift: false,
            }),
            Some(InputMsg::Commit)
        );
    }

    #[test]
    fn input_machine_edits_and_commits_text() {
        let machine = InputMachine;
        let mut runtime = Runtime::new(
            machine,
            InputContext {
                placeholder: "type here".into(),
                ..InputContext::default()
            },
            (),
        );

        runtime.send(InputMsg::Insert('a'));
        runtime.send(InputMsg::Insert('b'));
        runtime.send(InputMsg::MoveLeft);
        runtime.send(InputMsg::Insert('!'));
        runtime.send(InputMsg::Commit);

        assert_eq!(
            runtime.model(),
            InputState {
                value: "a!b".into(),
                cursor: 2,
                committed: Some("a!b".into()),
            }
        );
    }

    #[test]
    fn input_machine_renders_placeholder_and_value() {
        let machine = InputMachine;
        let mut runtime = Runtime::new(
            machine,
            InputContext {
                placeholder: "search".into(),
                width: 10,
                ..InputContext::default()
            },
            (),
        );

        let initial = runtime.render_ops(Rect::new(0, 0, 20, 3));
        assert!(initial.iter().any(|op| matches!(
            op,
            RenderOp::DrawText { content, .. } if content == "search"
        )));

        runtime.send(InputMsg::Insert('x'));
        let edited = runtime.render_ops(Rect::new(0, 0, 20, 3));
        assert!(edited.iter().any(|op| matches!(
            op,
            RenderOp::DrawText { content, .. } if content == "x"
        )));
    }

    #[test]
    fn input_machine_reports_cursor_position() {
        let machine = InputMachine;
        let mut runtime = Runtime::new(machine, InputContext::default(), ());
        runtime.send(InputMsg::Insert('a'));
        runtime.send(InputMsg::Insert('b'));
        runtime.send(InputMsg::MoveLeft);

        assert_eq!(runtime.cursor(Rect::new(0, 0, 20, 3)), Some((2, 1)));
    }
}
