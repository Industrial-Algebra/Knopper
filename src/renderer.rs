// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use crate::{PatchOp, RenderOp};

pub trait Renderer {
    type Error;

    fn apply(&mut self, patches: &[PatchOp]) -> Result<(), Self::Error>;
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MockRenderer {
    applied: Vec<PatchOp>,
}

impl MockRenderer {
    #[must_use]
    pub fn applied(&self) -> &[PatchOp] {
        &self.applied
    }
}

impl Renderer for MockRenderer {
    type Error = core::convert::Infallible;

    fn apply(&mut self, patches: &[PatchOp]) -> Result<(), Self::Error> {
        self.applied.extend_from_slice(patches);
        Ok(())
    }
}

pub fn render_once<R: Renderer>(renderer: &mut R, ops: &[RenderOp]) -> Result<(), R::Error> {
    let patches = ops.iter().cloned().map(PatchOp::Insert).collect::<Vec<_>>();
    renderer.apply(&patches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NodeId, Style, layout::Rect};

    #[test]
    fn mock_renderer_records_applied_patches() {
        let mut renderer = MockRenderer::default();
        let patches = vec![PatchOp::Insert(RenderOp::DrawText {
            id: NodeId::new(1),
            rect: Rect::new(0, 0, 2, 1),
            content: "ok".into(),
            style: Style::PLAIN,
        })];

        renderer
            .apply(&patches)
            .expect("mock renderer should not fail");
        assert_eq!(renderer.applied(), patches.as_slice());
    }

    #[test]
    fn render_once_inserts_all_render_ops() {
        let mut renderer = MockRenderer::default();
        let ops = vec![RenderOp::Annotate {
            id: NodeId::new(7),
            rect: Rect::new(0, 0, 4, 1),
            label: "meta".into(),
        }];

        render_once(&mut renderer, &ops).expect("mock renderer should not fail");
        assert_eq!(renderer.applied(), &[PatchOp::Insert(ops[0].clone())]);
    }
}
