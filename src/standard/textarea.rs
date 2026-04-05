use crate::{
    Color, Effect, Key, KeyEvent, LayoutNode, Machine, NodeId, Role, Scene, SceneBehavior,
    SizeConstraint, Style, find_node,
};
use cliffy_core::{Behavior, FromGeometric, GA3, IntoGeometric, behavior};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextareaState {
    pub value: String,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub preferred_col: Option<usize>,
    pub committed: Option<String>,
}

impl IntoGeometric for TextareaState {
    fn into_geometric(self) -> GA3 {
        GA3::zero()
    }
}

impl FromGeometric for TextareaState {
    fn from_geometric(_mv: &GA3) -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextareaMsg {
    Insert(char),
    Backspace,
    Newline,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Commit,
}

#[must_use]
pub fn textarea_key_msg(event: KeyEvent) -> Option<TextareaMsg> {
    match event.key {
        Key::Char(ch) if !event.ctrl && !event.alt => Some(TextareaMsg::Insert(ch)),
        Key::Backspace => Some(TextareaMsg::Backspace),
        Key::Enter if event.ctrl => Some(TextareaMsg::Commit),
        Key::Enter => Some(TextareaMsg::Newline),
        Key::Left => Some(TextareaMsg::MoveLeft),
        Key::Right => Some(TextareaMsg::MoveRight),
        Key::Up => Some(TextareaMsg::MoveUp),
        Key::Down => Some(TextareaMsg::MoveDown),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct TextareaContext {
    pub root_id: NodeId,
    pub content_id: NodeId,
    pub line_base_id: NodeId,
    pub width: u16,
    pub height: u16,
    pub placeholder: String,
}

impl Default for TextareaContext {
    fn default() -> Self {
        Self {
            root_id: NodeId::new(80_000),
            content_id: NodeId::new(80_001),
            line_base_id: NodeId::new(80_100),
            width: 24,
            height: 4,
            placeholder: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TextareaMachine;

impl TextareaMachine {
    #[must_use]
    pub fn key_msg(&self, event: KeyEvent) -> Option<TextareaMsg> {
        textarea_key_msg(event)
    }

    #[must_use]
    pub fn root_id(&self, ctx: &TextareaContext) -> NodeId {
        ctx.root_id
    }

    #[must_use]
    pub fn line_id(&self, ctx: &TextareaContext, index: usize) -> NodeId {
        NodeId::new(
            ctx.line_base_id
                .get()
                .saturating_add(u64::try_from(index).unwrap_or(u64::MAX)),
        )
    }
}

impl Machine for TextareaMachine {
    type Context = TextareaContext;
    type Msg = TextareaMsg;
    type Model = TextareaState;
    type Shared = ();

    fn init(&self, _ctx: &Self::Context) -> Self::Model {
        TextareaState::default()
    }

    fn update(
        &self,
        model: &mut Self::Model,
        msg: Self::Msg,
        _ctx: &Self::Context,
    ) -> Effect<Self::Msg> {
        match msg {
            TextareaMsg::Insert(ch) => {
                let idx = line_col_to_byte_index(&model.value, model.cursor_row, model.cursor_col);
                model.value.insert(idx, ch);
                model.cursor_col = model.cursor_col.saturating_add(1);
                model.preferred_col = Some(model.cursor_col);
            }
            TextareaMsg::Backspace => {
                if let Some(idx) =
                    previous_cursor_byte_index(&model.value, model.cursor_row, model.cursor_col)
                {
                    let current =
                        line_col_to_byte_index(&model.value, model.cursor_row, model.cursor_col);
                    model.value.replace_range(idx..current, "");
                    move_cursor_left(model);
                }
            }
            TextareaMsg::Newline => {
                let idx = line_col_to_byte_index(&model.value, model.cursor_row, model.cursor_col);
                model.value.insert(idx, '\n');
                model.cursor_row = model.cursor_row.saturating_add(1);
                model.cursor_col = 0;
                model.preferred_col = Some(0);
            }
            TextareaMsg::MoveLeft => move_cursor_left(model),
            TextareaMsg::MoveRight => move_cursor_right(model),
            TextareaMsg::MoveUp => move_cursor_vertical(model, true),
            TextareaMsg::MoveDown => move_cursor_vertical(model, false),
            TextareaMsg::Commit => {
                model.committed = Some(model.value.clone());
            }
        }
        clamp_cursor(model);
        Effect::None
    }

    fn project_once(
        &self,
        model: &Self::Model,
        _shared: &Self::Shared,
        ctx: &Self::Context,
    ) -> Scene<Self::Msg> {
        let lines = display_lines(model, ctx);
        Scene::border(
            ctx.root_id,
            Scene::sized(
                NodeId::new(ctx.root_id.get().saturating_add(10)),
                SizeConstraint::new(Some(ctx.width), Some(ctx.height)),
                Scene::column(
                    ctx.content_id,
                    lines
                        .into_iter()
                        .enumerate()
                        .map(|(index, line)| {
                            Scene::text(self.line_id(ctx, index), line)
                                .with_role(Role::Editor)
                                .with_style(if model.value.is_empty() {
                                    Style::PLAIN.fg(Color::Ansi(8))
                                } else {
                                    Style::PLAIN.fg(Color::Ansi(6)).bold()
                                })
                                .focusable()
                        })
                        .collect::<Vec<_>>(),
                ),
            ),
        )
        .with_style(Style::PLAIN.fg(Color::Ansi(8)))
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
            scene_for_model.set(TextareaMachine.project_once(next_model, &(), &ctx));
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
        let line = find_node(layout, self.line_id(ctx, model.cursor_row))?;
        let line_len = current_line(model).chars().count();
        let cursor_x = line.rect.x.saturating_add(
            u16::try_from(model.cursor_col)
                .unwrap_or(u16::MAX)
                .min(u16::try_from(line_len).unwrap_or(u16::MAX)),
        );
        Some((cursor_x, line.rect.y))
    }
}

fn display_lines(model: &TextareaState, ctx: &TextareaContext) -> Vec<String> {
    if model.value.is_empty() {
        return vec![ctx.placeholder.clone()];
    }

    model.value.lines().map(ToOwned::to_owned).collect()
}

fn lines(value: &str) -> Vec<&str> {
    let split: Vec<&str> = value.split('\n').collect();
    if split.is_empty() { vec![""] } else { split }
}

fn current_line(model: &TextareaState) -> &str {
    lines(&model.value)
        .get(model.cursor_row)
        .copied()
        .unwrap_or("")
}

fn clamp_cursor(model: &mut TextareaState) {
    let all_lines = lines(&model.value);
    model.cursor_row = model.cursor_row.min(all_lines.len().saturating_sub(1));
    let line_len = all_lines
        .get(model.cursor_row)
        .map(|line| line.chars().count())
        .unwrap_or(0);
    model.cursor_col = model.cursor_col.min(line_len);
}

fn line_col_to_byte_index(value: &str, row: usize, col: usize) -> usize {
    let all_lines = lines(value);
    let prefix_len: usize = all_lines
        .iter()
        .take(row)
        .map(|line| line.len().saturating_add(1))
        .sum();
    let line = all_lines.get(row).copied().unwrap_or("");
    prefix_len.saturating_add(char_to_byte_index(line, col))
}

fn char_to_byte_index(value: &str, char_index: usize) -> usize {
    value
        .char_indices()
        .nth(char_index)
        .map_or(value.len(), |(idx, _)| idx)
}

fn previous_cursor_byte_index(value: &str, row: usize, col: usize) -> Option<usize> {
    if row == 0 && col == 0 {
        None
    } else if col > 0 {
        Some(line_col_to_byte_index(value, row, col - 1))
    } else {
        let previous_line = lines(value).get(row.saturating_sub(1))?.chars().count();
        Some(line_col_to_byte_index(
            value,
            row.saturating_sub(1),
            previous_line,
        ))
    }
}

fn move_cursor_left(model: &mut TextareaState) {
    if model.cursor_col > 0 {
        model.cursor_col -= 1;
    } else if model.cursor_row > 0 {
        model.cursor_row -= 1;
        model.cursor_col = lines(&model.value)
            .get(model.cursor_row)
            .map(|line| line.chars().count())
            .unwrap_or(0);
    }
    model.preferred_col = Some(model.cursor_col);
}

fn move_cursor_right(model: &mut TextareaState) {
    let all_lines = lines(&model.value);
    let line_len = all_lines
        .get(model.cursor_row)
        .map(|line| line.chars().count())
        .unwrap_or(0);
    if model.cursor_col < line_len {
        model.cursor_col += 1;
    } else if model.cursor_row + 1 < all_lines.len() {
        model.cursor_row += 1;
        model.cursor_col = 0;
    }
    model.preferred_col = Some(model.cursor_col);
}

fn move_cursor_vertical(model: &mut TextareaState, up: bool) {
    let all_lines = lines(&model.value);
    if up {
        model.cursor_row = model.cursor_row.saturating_sub(1);
    } else {
        model.cursor_row = (model.cursor_row + 1).min(all_lines.len().saturating_sub(1));
    }
    let preferred = model.preferred_col.unwrap_or(model.cursor_col);
    let line_len = all_lines
        .get(model.cursor_row)
        .map(|line| line.chars().count())
        .unwrap_or(0);
    model.cursor_col = preferred.min(line_len);
    model.preferred_col = Some(preferred);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderOp, Runtime, layout::Rect};

    fn ctx() -> TextareaContext {
        TextareaContext {
            width: 16,
            height: 4,
            placeholder: "type here".into(),
            ..TextareaContext::default()
        }
    }

    #[test]
    fn textarea_key_binding_maps_multiline_editor_keys() {
        assert_eq!(
            textarea_key_msg(KeyEvent {
                key: Key::Char('a'),
                ctrl: false,
                alt: false,
                shift: false,
            }),
            Some(TextareaMsg::Insert('a'))
        );
        assert_eq!(
            textarea_key_msg(KeyEvent {
                key: Key::Enter,
                ctrl: false,
                alt: false,
                shift: false,
            }),
            Some(TextareaMsg::Newline)
        );
        assert_eq!(
            textarea_key_msg(KeyEvent {
                key: Key::Enter,
                ctrl: true,
                alt: false,
                shift: false,
            }),
            Some(TextareaMsg::Commit)
        );
        assert_eq!(
            textarea_key_msg(KeyEvent {
                key: Key::Up,
                ctrl: false,
                alt: false,
                shift: false,
            }),
            Some(TextareaMsg::MoveUp)
        );
    }

    #[test]
    fn textarea_machine_edits_multiple_lines_and_commits() {
        let machine = TextareaMachine;
        let mut runtime = Runtime::new(machine, ctx(), ());

        runtime.send(TextareaMsg::Insert('a'));
        runtime.send(TextareaMsg::Insert('b'));
        runtime.send(TextareaMsg::Newline);
        runtime.send(TextareaMsg::Insert('c'));
        runtime.send(TextareaMsg::Commit);

        assert_eq!(
            runtime.model(),
            TextareaState {
                value: "ab\nc".into(),
                cursor_row: 1,
                cursor_col: 1,
                preferred_col: Some(1),
                committed: Some("ab\nc".into()),
            }
        );
    }

    #[test]
    fn textarea_machine_moves_cursor_across_lines() {
        let machine = TextareaMachine;
        let mut runtime = Runtime::new(machine, ctx(), ());

        runtime.send(TextareaMsg::Insert('a'));
        runtime.send(TextareaMsg::Insert('b'));
        runtime.send(TextareaMsg::Newline);
        runtime.send(TextareaMsg::Insert('x'));
        runtime.send(TextareaMsg::MoveUp);

        assert_eq!(runtime.model().cursor_row, 0);
        assert_eq!(runtime.model().cursor_col, 1);

        runtime.send(TextareaMsg::MoveDown);
        assert_eq!(runtime.model().cursor_row, 1);
        assert_eq!(runtime.model().cursor_col, 1);
    }

    #[test]
    fn textarea_machine_renders_placeholder_and_multiline_content() {
        let machine = TextareaMachine;
        let mut runtime = Runtime::new(machine, ctx(), ());

        assert!(
            runtime.render_ops(Rect::new(0, 0, 20, 6)).iter().any(
                |op| matches!(op, RenderOp::DrawText { content, .. } if content == "type here")
            )
        );

        runtime.send(TextareaMsg::Insert('a'));
        runtime.send(TextareaMsg::Newline);
        runtime.send(TextareaMsg::Insert('b'));

        let ops = runtime.render_ops(Rect::new(0, 0, 20, 6));
        assert!(
            ops.iter()
                .any(|op| matches!(op, RenderOp::DrawText { content, .. } if content == "a"))
        );
        assert!(
            ops.iter()
                .any(|op| matches!(op, RenderOp::DrawText { content, .. } if content == "b"))
        );
    }

    #[test]
    fn textarea_machine_reports_cursor_position() {
        let machine = TextareaMachine;
        let mut runtime = Runtime::new(machine, ctx(), ());

        runtime.send(TextareaMsg::Insert('a'));
        runtime.send(TextareaMsg::Insert('b'));
        runtime.send(TextareaMsg::Newline);
        runtime.send(TextareaMsg::Insert('c'));

        assert_eq!(runtime.cursor(Rect::new(0, 0, 20, 6)), Some((2, 2)));
    }
}
