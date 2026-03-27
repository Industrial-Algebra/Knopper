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
    Annotate {
        id: NodeId,
        rect: Rect,
        label: String,
    },
    ClearNode {
        id: NodeId,
    },
    ClearRect {
        rect: Rect,
    },
}

pub trait TerminalBackend {
    type Error;

    fn execute(&mut self, commands: &[BackendCommand]) -> Result<(), Self::Error>;
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MockBackend {
    executed: Vec<BackendCommand>,
}

impl MockBackend {
    #[must_use]
    pub fn executed(&self) -> &[BackendCommand] {
        &self.executed
    }
}

impl TerminalBackend for MockBackend {
    type Error = core::convert::Infallible;

    fn execute(&mut self, commands: &[BackendCommand]) -> Result<(), Self::Error> {
        self.executed.extend_from_slice(commands);
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
        RenderOp::DrawText { id, .. } | RenderOp::Annotate { id, .. } => *id,
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
        RenderOp::Annotate { id, rect, label } => BackendCommand::Annotate {
            id: *id,
            rect: *rect,
            label: label.clone(),
        },
    }
}

fn clear_command(op: &RenderOp) -> BackendCommand {
    match op {
        RenderOp::DrawText { rect, .. } | RenderOp::Annotate { rect, .. } => {
            BackendCommand::ClearRect { rect: *rect }
        }
    }
}

#[cfg(feature = "notcurses")]
pub mod notcurses {
    use super::{BackendCommand, TerminalBackend};
    use crate::layout::Rect;

    #[derive(Debug, Default)]
    pub struct NotcursesBackend {
        last_known_regions: Vec<Rect>,
    }

    impl NotcursesBackend {
        #[must_use]
        pub fn new() -> Self {
            Self::default()
        }

        #[must_use]
        pub fn tracked_regions(&self) -> &[Rect] {
            &self.last_known_regions
        }
    }

    impl TerminalBackend for NotcursesBackend {
        type Error = core::convert::Infallible;

        fn execute(&mut self, commands: &[BackendCommand]) -> Result<(), Self::Error> {
            for command in commands {
                match command {
                    BackendCommand::DrawText { rect, .. }
                    | BackendCommand::Annotate { rect, .. }
                    | BackendCommand::ClearRect { rect } => self.last_known_regions.push(*rect),
                    BackendCommand::ClearNode { .. } => {}
                }
            }
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
    fn mock_backend_records_executed_commands() {
        let mut backend = MockBackend::default();
        let commands = vec![BackendCommand::ClearNode { id: NodeId::new(9) }];

        backend
            .execute(&commands)
            .expect("mock backend should not fail");
        assert_eq!(backend.executed(), commands.as_slice());
    }
}
