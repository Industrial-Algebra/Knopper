use crate::{NodeId, PatchOp, RenderOp, layout::Rect, style::Style};

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
pub fn backend_commands(patches: &[PatchOp]) -> Vec<BackendCommand> {
    patches
        .iter()
        .map(|patch| match patch {
            PatchOp::Insert(op) | PatchOp::Update(op) => match op {
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
            },
            PatchOp::Remove(id) => BackendCommand::ClearNode { id: *id },
        })
        .collect()
}

#[cfg(feature = "notcurses")]
pub mod notcurses {
    use super::{BackendCommand, TerminalBackend};

    #[derive(Debug, Default)]
    pub struct NotcursesBackend;

    impl TerminalBackend for NotcursesBackend {
        type Error = core::convert::Infallible;

        fn execute(&mut self, _commands: &[BackendCommand]) -> Result<(), Self::Error> {
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
            backend_commands(&patches),
            vec![
                BackendCommand::DrawText {
                    id: NodeId::new(1),
                    rect: Rect::new(0, 0, 2, 1),
                    content: "ok".into(),
                    style: Style::PLAIN,
                },
                BackendCommand::ClearNode { id: NodeId::new(2) },
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
