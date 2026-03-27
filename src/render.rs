use crate::{
    NodeId, Style,
    layout::{LayoutKind, LayoutNode, Rect},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderOp {
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
}

#[must_use]
pub fn render_ops(layout: &LayoutNode) -> Vec<RenderOp> {
    let mut ops = Vec::new();
    collect_ops(layout, &mut ops);
    ops
}

fn collect_ops(layout: &LayoutNode, ops: &mut Vec<RenderOp>) {
    match &layout.kind {
        LayoutKind::Empty => {}
        LayoutKind::Text { content } => ops.push(RenderOp::DrawText {
            id: layout.id,
            rect: layout.rect,
            content: content
                .chars()
                .take(usize::from(layout.rect.width))
                .collect(),
            style: layout.style,
        }),
        LayoutKind::Row { children } | LayoutKind::Column { children } => {
            for child in children {
                collect_ops(child, ops);
            }
        }
        LayoutKind::Padding { child } | LayoutKind::Viewport { child } => collect_ops(child, ops),
        LayoutKind::Border { child } => {
            ops.push(RenderOp::DrawBorder {
                id: layout.id,
                rect: layout.rect,
                style: layout.style,
            });
            collect_ops(child, ops);
        }
        LayoutKind::Annotated { label, child } => {
            ops.push(RenderOp::Annotate {
                id: layout.id,
                rect: layout.rect,
                label: label.clone(),
            });
            collect_ops(child, ops);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Scene,
        layout::{Rect, resolve_layout},
        scene::Padding,
    };

    #[test]
    fn lowers_layout_tree_to_text_ops() {
        let scene = Scene::<()>::row(
            1_u64,
            vec![Scene::text(2_u64, "abc"), Scene::text(3_u64, "de")],
        );
        let layout = resolve_layout(&scene, Rect::new(0, 0, 20, 1));
        let ops = render_ops(&layout);

        assert_eq!(
            ops,
            vec![
                RenderOp::DrawText {
                    id: NodeId::new(2),
                    rect: Rect::new(0, 0, 3, 1),
                    content: "abc".into(),
                    style: Style::PLAIN,
                },
                RenderOp::DrawText {
                    id: NodeId::new(3),
                    rect: Rect::new(3, 0, 2, 1),
                    content: "de".into(),
                    style: Style::PLAIN,
                },
            ]
        );
    }

    #[test]
    fn text_rendering_is_clipped_to_layout_width() {
        let scene = Scene::<()>::viewport(
            1_u64,
            Scene::padding(2_u64, Padding::all(0), Scene::text(3_u64, "abcdef")),
        );
        let layout = resolve_layout(&scene, Rect::new(0, 0, 4, 1));
        let ops = render_ops(&layout);
        assert_eq!(
            ops,
            vec![RenderOp::DrawText {
                id: NodeId::new(3),
                rect: Rect::new(0, 0, 4, 1),
                content: "abcd".into(),
                style: Style::PLAIN,
            }]
        );
    }

    #[test]
    fn bordered_nodes_emit_border_ops_before_child_ops() {
        let scene = Scene::<()>::border(1_u64, Scene::text(2_u64, "abc"));
        let layout = resolve_layout(&scene, Rect::new(0, 0, 5, 3));
        let ops = render_ops(&layout);

        assert_eq!(
            ops,
            vec![
                RenderOp::DrawBorder {
                    id: NodeId::new(1),
                    rect: Rect::new(0, 0, 5, 3),
                    style: Style::PLAIN,
                },
                RenderOp::DrawText {
                    id: NodeId::new(2),
                    rect: Rect::new(1, 1, 3, 1),
                    content: "abc".into(),
                    style: Style::PLAIN,
                },
            ]
        );
    }

    #[test]
    fn annotated_nodes_emit_annotation_ops_before_child_ops() {
        let scene = Scene::<()>::annotated(1_u64, "debug", Scene::text(2_u64, "abc"));
        let layout = resolve_layout(&scene, Rect::new(0, 0, 5, 1));
        let ops = render_ops(&layout);

        assert_eq!(
            ops,
            vec![
                RenderOp::Annotate {
                    id: NodeId::new(1),
                    rect: Rect::new(0, 0, 5, 1),
                    label: "debug".into(),
                },
                RenderOp::DrawText {
                    id: NodeId::new(2),
                    rect: Rect::new(0, 0, 3, 1),
                    content: "abc".into(),
                    style: Style::PLAIN,
                },
            ]
        );
    }
}
