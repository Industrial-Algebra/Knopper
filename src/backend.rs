// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use crate::{NodeId, PatchOp, RenderOp, layout::Rect, style::Style};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendCommand {
    DrawText {
        id: NodeId,
        rect: Rect,
        content: String,
        style: Style,
    },
    DrawBorder {
        id: NodeId,
        rect: Rect,
        style: Style,
    },
    Annotate {
        id: NodeId,
        rect: Rect,
        label: String,
    },
    SetCursor {
        id: NodeId,
        position: Option<(u16, u16)>,
    },
    ClearNode {
        id: NodeId,
    },
    ClearRect {
        rect: Rect,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendEntry {
    Text {
        rect: Rect,
        content: String,
        style: Style,
    },
    Border {
        rect: Rect,
        style: Style,
    },
    Annotation {
        rect: Rect,
        label: String,
    },
}

impl BackendEntry {
    #[must_use]
    pub fn rect(&self) -> Rect {
        match self {
            Self::Text { rect, .. } | Self::Border { rect, .. } | Self::Annotation { rect, .. } => {
                *rect
            }
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BackendState {
    nodes: BTreeMap<NodeId, BackendEntry>,
    cursor: Option<(u16, u16)>,
}

impl BackendState {
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&BackendEntry> {
        self.nodes.get(&id)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    #[must_use]
    pub fn cursor(&self) -> Option<(u16, u16)> {
        self.cursor
    }

    pub fn apply(&mut self, commands: &[BackendCommand]) {
        for command in commands {
            match command {
                BackendCommand::DrawText {
                    id,
                    rect,
                    content,
                    style,
                } => {
                    self.nodes.insert(
                        *id,
                        BackendEntry::Text {
                            rect: *rect,
                            content: content.clone(),
                            style: *style,
                        },
                    );
                }
                BackendCommand::DrawBorder { id, rect, style } => {
                    self.nodes.insert(
                        *id,
                        BackendEntry::Border {
                            rect: *rect,
                            style: *style,
                        },
                    );
                }
                BackendCommand::Annotate { id, rect, label } => {
                    self.nodes.insert(
                        *id,
                        BackendEntry::Annotation {
                            rect: *rect,
                            label: label.clone(),
                        },
                    );
                }
                BackendCommand::SetCursor { position, .. } => {
                    self.cursor = *position;
                }
                BackendCommand::ClearNode { id } => {
                    self.nodes.remove(id);
                }
                BackendCommand::ClearRect { .. } => {}
            }
        }
    }
}

pub trait TerminalBackend {
    type Error;

    fn execute(&mut self, commands: &[BackendCommand]) -> Result<(), Self::Error>;

    fn sync_order(&mut self, _order: &[NodeId]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn reset(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MockBackend {
    executed: Vec<BackendCommand>,
    state: BackendState,
}

impl MockBackend {
    #[must_use]
    pub fn executed(&self) -> &[BackendCommand] {
        &self.executed
    }

    #[must_use]
    pub fn state(&self) -> &BackendState {
        &self.state
    }
}

impl TerminalBackend for MockBackend {
    type Error = core::convert::Infallible;

    fn execute(&mut self, commands: &[BackendCommand]) -> Result<(), Self::Error> {
        self.executed.extend_from_slice(commands);
        self.state.apply(commands);
        Ok(())
    }
}

#[must_use]
pub fn backend_commands(previous: &[RenderOp], patches: &[PatchOp]) -> Vec<BackendCommand> {
    let previous_by_id: BTreeMap<NodeId, &RenderOp> =
        previous.iter().map(|op| (op_id(op), op)).collect();

    patches
        .iter()
        .flat_map(|patch| match patch {
            PatchOp::Insert(op) => vec![draw_command(op)],
            PatchOp::Update(op) => match previous_by_id.get(&op_id(op)) {
                Some(previous) => vec![clear_command(previous), draw_command(op)],
                None => vec![draw_command(op)],
            },
            PatchOp::Remove(id) => previous_by_id
                .get(id)
                .map(|op| vec![clear_command(op), BackendCommand::ClearNode { id: *id }])
                .unwrap_or_else(|| vec![BackendCommand::ClearNode { id: *id }]),
        })
        .collect()
}

fn op_id(op: &RenderOp) -> NodeId {
    match op {
        RenderOp::DrawText { id, .. }
        | RenderOp::DrawBorder { id, .. }
        | RenderOp::Annotate { id, .. }
        | RenderOp::SetCursor { id, .. } => *id,
    }
}

fn draw_command(op: &RenderOp) -> BackendCommand {
    match op {
        RenderOp::DrawText {
            id,
            rect,
            content,
            style,
        } => BackendCommand::DrawText {
            id: *id,
            rect: *rect,
            content: content.clone(),
            style: *style,
        },
        RenderOp::DrawBorder { id, rect, style } => BackendCommand::DrawBorder {
            id: *id,
            rect: *rect,
            style: *style,
        },
        RenderOp::Annotate { id, rect, label } => BackendCommand::Annotate {
            id: *id,
            rect: *rect,
            label: label.clone(),
        },
        RenderOp::SetCursor { id, position } => BackendCommand::SetCursor {
            id: *id,
            position: *position,
        },
    }
}

fn clear_command(op: &RenderOp) -> BackendCommand {
    match op {
        RenderOp::DrawText { rect, .. }
        | RenderOp::DrawBorder { rect, .. }
        | RenderOp::Annotate { rect, .. } => BackendCommand::ClearRect { rect: *rect },
        RenderOp::SetCursor { .. } => BackendCommand::SetCursor {
            id: NodeId::new(0),
            position: None,
        },
    }
}

#[cfg(feature = "notcurses")]
pub mod notcurses {
    use super::{BackendCommand, BackendState, TerminalBackend};
    use crate::{
        NodeId,
        layout::Rect,
        style::{Color, Style as KnopperStyle},
    };
    use notcurses::{Input as NcInput, Notcurses, NotcursesError, Plane, Style as NcStyle};
    use std::collections::BTreeMap;

    #[derive(Debug)]
    struct SurfaceRecord {
        plane: Plane,
        rect: Rect,
    }

    #[derive(Debug)]
    pub struct NotcursesBackend {
        nc: Notcurses,
        root: Plane,
        surfaces: BTreeMap<NodeId, SurfaceRecord>,
        state: BackendState,
        command_log: Vec<BackendCommand>,
        cursor_visible: bool,
    }

    impl NotcursesBackend {
        pub fn new() -> Result<Self, NotcursesError> {
            let nc = Notcurses::new()?;
            let root = Plane::new(&nc)?;
            Ok(Self {
                nc,
                root,
                surfaces: BTreeMap::new(),
                state: BackendState::default(),
                command_log: Vec::new(),
                cursor_visible: false,
            })
        }

        #[must_use]
        pub fn state(&self) -> &BackendState {
            &self.state
        }

        #[must_use]
        pub fn command_log(&self) -> &[BackendCommand] {
            &self.command_log
        }

        #[must_use]
        pub fn surface_count(&self) -> usize {
            self.surfaces.len()
        }

        pub fn read_event(&self) -> Result<NcInput, NotcursesError> {
            self.nc.get_event()
        }

        fn apply_style(plane: &mut Plane, style: KnopperStyle) {
            let mut nc_style = NcStyle::None;
            if style.emphasis.bold {
                nc_style.set(NcStyle::Bold);
            }
            if style.emphasis.italic {
                nc_style.set(NcStyle::Italic);
            }
            if style.emphasis.underlined {
                nc_style.set(NcStyle::Underline);
            }
            plane.set_styles(nc_style);

            match style.fg {
                None | Some(Color::Default) => {
                    let _ = plane.unset_fg();
                }
                Some(Color::Ansi(index)) => plane.set_fg_palindex(index),
            }

            match style.bg {
                None | Some(Color::Default) => {
                    let _ = plane.unset_bg();
                }
                Some(Color::Ansi(index)) => plane.set_bg_palindex(index),
            }
        }

        fn clear_rect(&mut self, rect: Rect) -> Result<(), NotcursesError> {
            let (cols, rows): (u32, u32) = self.root.size().into();
            let max_y = u16::try_from(rows).unwrap_or(u16::MAX);
            let max_x = u16::try_from(cols).unwrap_or(u16::MAX);

            if rect.x >= max_x || rect.y >= max_y {
                return Ok(());
            }

            let width = rect.width.min(max_x.saturating_sub(rect.x));
            let height = rect.height.min(max_y.saturating_sub(rect.y));
            if width == 0 || height == 0 {
                return Ok(());
            }

            self.root.erase_region(
                Some(u32::from(rect.x)),
                Some(u32::from(rect.y)),
                i32::from(width),
                i32::from(height),
            )?;
            Ok(())
        }

        fn ensure_surface(
            &mut self,
            id: NodeId,
            rect: Rect,
        ) -> Result<&mut SurfaceRecord, NotcursesError> {
            let needs_new = self
                .surfaces
                .get(&id)
                .is_none_or(|surface| surface.rect.width == 0 || surface.rect.height == 0);

            if needs_new {
                let plane = self.root.new_child_sized_at(
                    (u32::from(rect.width.max(1)), u32::from(rect.height.max(1))),
                    (u32::from(rect.x), u32::from(rect.y)),
                )?;
                self.surfaces.insert(id, SurfaceRecord { plane, rect });
            }

            let surface = self
                .surfaces
                .get_mut(&id)
                .expect("surface must exist after insertion");

            if surface.rect != rect {
                surface
                    .plane
                    .move_to((u32::from(rect.x), u32::from(rect.y)))?;
                surface
                    .plane
                    .resize_simple((u32::from(rect.width.max(1)), u32::from(rect.height.max(1))))?;
                surface.rect = rect;
            }

            surface.plane.erase();
            Ok(surface)
        }

        fn fill_surface(
            plane: &mut Plane,
            rect: Rect,
            style: KnopperStyle,
        ) -> Result<(), NotcursesError> {
            if style.bg.is_none() {
                return Ok(());
            }
            let blank = " ".repeat(usize::from(rect.width.max(1)));
            for y in 0..rect.height.max(1) {
                plane.putstr_at((0_u32, u32::from(y)), &blank)?;
            }
            Ok(())
        }

        fn draw_text(
            &mut self,
            id: NodeId,
            rect: Rect,
            content: &str,
            style: KnopperStyle,
        ) -> Result<(), NotcursesError> {
            let surface = self.ensure_surface(id, rect)?;
            Self::apply_style(&mut surface.plane, style);
            Self::fill_surface(&mut surface.plane, rect, style)?;
            surface.plane.putstr_at((0_u32, 0_u32), content)?;
            surface.plane.move_top();
            Ok(())
        }

        fn draw_border(
            &mut self,
            id: NodeId,
            rect: Rect,
            style: KnopperStyle,
        ) -> Result<(), NotcursesError> {
            let surface = self.ensure_surface(id, rect)?;
            Self::apply_style(&mut surface.plane, style);
            Self::fill_surface(&mut surface.plane, rect, style)?;
            let w = u32::from(rect.width.max(2));
            let h = u32::from(rect.height.max(2));

            surface.plane.putstr_at((0_u32, 0_u32), "┌")?;
            surface.plane.putstr_at((w - 1, 0_u32), "┐")?;
            surface.plane.putstr_at((0_u32, h - 1), "└")?;
            surface.plane.putstr_at((w - 1, h - 1), "┘")?;

            for x in 1..w.saturating_sub(1) {
                surface.plane.putstr_at((x, 0_u32), "─")?;
                surface.plane.putstr_at((x, h - 1), "─")?;
            }
            for y in 1..h.saturating_sub(1) {
                surface.plane.putstr_at((0_u32, y), "│")?;
                surface.plane.putstr_at((w - 1, y), "│")?;
            }

            surface.plane.move_top();
            Ok(())
        }

        fn draw_annotation(
            &mut self,
            id: NodeId,
            rect: Rect,
            label: &str,
        ) -> Result<(), NotcursesError> {
            let surface = self.ensure_surface(id, rect)?;
            Self::apply_style(&mut surface.plane, KnopperStyle::PLAIN);
            surface.plane.putstr_at((0_u32, 0_u32), label)?;
            surface.plane.move_top();
            Ok(())
        }

        fn clear_node(&mut self, id: NodeId) {
            self.surfaces.remove(&id);
        }
    }

    impl TerminalBackend for NotcursesBackend {
        type Error = NotcursesError;

        fn execute(&mut self, commands: &[BackendCommand]) -> Result<(), Self::Error> {
            for command in commands {
                let result = match command {
                    BackendCommand::DrawText {
                        id,
                        rect,
                        content,
                        style,
                    } => self.draw_text(*id, *rect, content, *style),
                    BackendCommand::DrawBorder { id, rect, style } => {
                        self.draw_border(*id, *rect, *style)
                    }
                    BackendCommand::Annotate { id, rect, label } => {
                        self.draw_annotation(*id, *rect, label)
                    }
                    BackendCommand::SetCursor { position, .. } => match position {
                        Some((x, y)) => {
                            let result = self.nc.cursor_enable((u32::from(*y), u32::from(*x)));
                            if result.is_ok() {
                                self.cursor_visible = true;
                            }
                            result
                        }
                        None => {
                            if self.cursor_visible {
                                self.cursor_visible = false;
                                if let Err(error) = self.nc.cursor_disable() {
                                    eprintln!("notcurses cursor disable ignored: {error}");
                                }
                            }
                            Ok(())
                        }
                    },
                    BackendCommand::ClearRect { rect } => self.clear_rect(*rect),
                    BackendCommand::ClearNode { id } => {
                        self.clear_node(*id);
                        Ok(())
                    }
                };

                if let Err(error) = result {
                    eprintln!("notcurses command failed: {command:?}: {error}");
                    return Err(error);
                }
            }

            self.root.render()?;
            self.command_log.extend_from_slice(commands);
            self.state.apply(commands);
            Ok(())
        }

        fn sync_order(&mut self, order: &[NodeId]) -> Result<(), Self::Error> {
            for id in order {
                if let Some(surface) = self.surfaces.get_mut(id) {
                    surface.plane.move_top();
                }
            }
            self.root.render()?;
            Ok(())
        }

        fn reset(&mut self) -> Result<(), Self::Error> {
            self.surfaces.clear();
            self.command_log.clear();
            self.cursor_visible = false;
            self.root.erase();
            self.root.render()?;
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), Self::Error> {
            self.surfaces.clear();
            self.cursor_visible = false;
            self.root.erase();
            let _ = self.nc.cursor_disable();
            self.root.render()?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Rect, Style};

    #[test]
    fn translates_patch_ops_to_backend_commands() {
        let previous = vec![RenderOp::DrawText {
            id: NodeId::new(2),
            rect: Rect::new(0, 0, 4, 1),
            content: "gone".into(),
            style: Style::PLAIN,
        }];
        let patches = vec![
            PatchOp::Insert(RenderOp::DrawText {
                id: NodeId::new(1),
                rect: Rect::new(0, 0, 2, 1),
                content: "ok".into(),
                style: Style::PLAIN,
            }),
            PatchOp::Remove(NodeId::new(2)),
        ];

        assert_eq!(
            backend_commands(&previous, &patches),
            vec![
                BackendCommand::DrawText {
                    id: NodeId::new(1),
                    rect: Rect::new(0, 0, 2, 1),
                    content: "ok".into(),
                    style: Style::PLAIN,
                },
                BackendCommand::ClearRect {
                    rect: Rect::new(0, 0, 4, 1),
                },
                BackendCommand::ClearNode { id: NodeId::new(2) },
            ]
        );
    }

    #[test]
    fn translates_border_and_cursor_ops() {
        let next = vec![
            RenderOp::DrawBorder {
                id: NodeId::new(1),
                rect: Rect::new(0, 0, 5, 3),
                style: Style::PLAIN,
            },
            RenderOp::SetCursor {
                id: NodeId::new(99),
                position: Some((2, 1)),
            },
        ];

        assert_eq!(
            backend_commands(
                &[],
                &next.into_iter().map(PatchOp::Insert).collect::<Vec<_>>()
            ),
            vec![
                BackendCommand::DrawBorder {
                    id: NodeId::new(1),
                    rect: Rect::new(0, 0, 5, 3),
                    style: Style::PLAIN,
                },
                BackendCommand::SetCursor {
                    id: NodeId::new(99),
                    position: Some((2, 1)),
                },
            ]
        );
    }

    #[test]
    fn updates_clear_then_redraw() {
        let previous = vec![RenderOp::DrawText {
            id: NodeId::new(1),
            rect: Rect::new(0, 0, 2, 1),
            content: "ok".into(),
            style: Style::PLAIN,
        }];
        let patches = vec![PatchOp::Update(RenderOp::DrawText {
            id: NodeId::new(1),
            rect: Rect::new(0, 0, 3, 1),
            content: "new".into(),
            style: Style::PLAIN,
        })];

        assert_eq!(
            backend_commands(&previous, &patches),
            vec![
                BackendCommand::ClearRect {
                    rect: Rect::new(0, 0, 2, 1),
                },
                BackendCommand::DrawText {
                    id: NodeId::new(1),
                    rect: Rect::new(0, 0, 3, 1),
                    content: "new".into(),
                    style: Style::PLAIN,
                },
            ]
        );
    }

    #[test]
    fn backend_state_tracks_nodes_by_identity() {
        let mut state = BackendState::default();
        state.apply(&[
            BackendCommand::DrawText {
                id: NodeId::new(1),
                rect: Rect::new(0, 0, 2, 1),
                content: "ok".into(),
                style: Style::PLAIN,
            },
            BackendCommand::DrawBorder {
                id: NodeId::new(2),
                rect: Rect::new(0, 1, 4, 3),
                style: Style::PLAIN,
            },
        ]);

        assert_eq!(state.len(), 2);
        assert_eq!(
            state.get(NodeId::new(1)),
            Some(&BackendEntry::Text {
                rect: Rect::new(0, 0, 2, 1),
                content: "ok".into(),
                style: Style::PLAIN,
            })
        );
        assert_eq!(
            state.get(NodeId::new(2)),
            Some(&BackendEntry::Border {
                rect: Rect::new(0, 1, 4, 3),
                style: Style::PLAIN,
            })
        );
    }

    #[test]
    fn backend_state_tracks_cursor() {
        let mut state = BackendState::default();
        state.apply(&[BackendCommand::SetCursor {
            id: NodeId::new(9),
            position: Some((3, 4)),
        }]);
        assert_eq!(state.cursor(), Some((3, 4)));
    }

    #[test]
    fn backend_state_removes_nodes_on_clear_node() {
        let mut state = BackendState::default();
        state.apply(&[BackendCommand::DrawText {
            id: NodeId::new(1),
            rect: Rect::new(0, 0, 2, 1),
            content: "ok".into(),
            style: Style::PLAIN,
        }]);
        state.apply(&[BackendCommand::ClearRect {
            rect: Rect::new(0, 0, 2, 1),
        }]);
        assert_eq!(state.len(), 1);

        state.apply(&[BackendCommand::ClearNode { id: NodeId::new(1) }]);
        assert!(state.is_empty());
    }

    #[test]
    fn mock_backend_records_executed_commands_and_updates_state() {
        let mut backend = MockBackend::default();
        let commands = vec![
            BackendCommand::DrawText {
                id: NodeId::new(9),
                rect: Rect::new(1, 2, 3, 1),
                content: "hey".into(),
                style: Style::PLAIN,
            },
            BackendCommand::SetCursor {
                id: NodeId::new(10),
                position: Some((1, 2)),
            },
        ];

        backend
            .execute(&commands)
            .expect("mock backend should not fail");
        assert_eq!(backend.executed(), commands.as_slice());
        assert_eq!(
            backend.state().get(NodeId::new(9)),
            Some(&BackendEntry::Text {
                rect: Rect::new(1, 2, 3, 1),
                content: "hey".into(),
                style: Style::PLAIN,
            })
        );
        assert_eq!(backend.state().cursor(), Some((1, 2)));
    }
}
