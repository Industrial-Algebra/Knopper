use crate::{
    NodeId, Style,
    layout::{LayoutKind, LayoutNode, Rect},
    scene::ScrollOffset,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct RenderContext {
    scroll: ScrollOffset,
    clip: Option<Rect>,
}

#[must_use]
pub fn render_ops(layout: &LayoutNode) -> Vec<RenderOp> {
    let mut ops = Vec::new();
    collect_ops(layout, &mut ops, RenderContext::default());
    ops
}

fn collect_ops(layout: &LayoutNode, ops: &mut Vec<RenderOp>, context: RenderContext) {
    match &layout.kind {
        LayoutKind::Empty => {}
        LayoutKind::Text { content } => {
            if let Some((rect, visible)) = clip_text(layout.rect, content, context) {
                ops.push(RenderOp::DrawText {
                    id: layout.id,
                    rect,
                    content: visible,
                    style: layout.style,
                });
            }
        }
        LayoutKind::Row { children }
        | LayoutKind::Column { children }
        | LayoutKind::Stack { children } => {
            for child in children {
                collect_ops(child, ops, context);
            }
        }
        LayoutKind::Padding { child } | LayoutKind::Sized { child } => {
            collect_ops(child, ops, context)
        }
        LayoutKind::Viewport { child } => collect_ops(
            child,
            ops,
            RenderContext {
                clip: combine_clip(context.clip, translate_rect(layout.rect, context.scroll)),
                ..context
            },
        ),
        LayoutKind::Scroll { offset, child } => collect_ops(
            child,
            ops,
            RenderContext {
                scroll: ScrollOffset {
                    x: context.scroll.x.saturating_add(offset.x),
                    y: context.scroll.y.saturating_add(offset.y),
                },
                clip: combine_clip(context.clip, Some(layout.rect)),
            },
        ),
        LayoutKind::Border { child } => {
            if let Some(rect) = translate_rect(layout.rect, context.scroll)
                .and_then(|rect| apply_clip(rect, context.clip))
            {
                ops.push(RenderOp::DrawBorder {
                    id: layout.id,
                    rect,
                    style: layout.style,
                });
            }
            collect_ops(child, ops, context);
        }
        LayoutKind::Annotated { label, child } => {
            if let Some(rect) = translate_rect(layout.rect, context.scroll)
                .and_then(|rect| apply_clip(rect, context.clip))
            {
                ops.push(RenderOp::Annotate {
                    id: layout.id,
                    rect,
                    label: label.clone(),
                });
            }
            collect_ops(child, ops, context);
        }
    }
}

fn clip_text(rect: Rect, content: &str, context: RenderContext) -> Option<(Rect, String)> {
    let translated_y = rect.y.checked_sub(context.scroll.y)?;
    let left = i32::from(rect.x) - i32::from(context.scroll.x);
    let right = i32::from(rect.x) + i32::from(rect.width) - i32::from(context.scroll.x);

    if right <= 0 {
        return None;
    }

    let visible_left = left.max(0);
    let visible_width = u16::try_from(right.saturating_sub(visible_left)).ok()?;
    let translated_x = u16::try_from(visible_left).ok()?;
    let mut visible_rect = Rect::new(translated_x, translated_y, visible_width, rect.height);
    let mut skip = usize::try_from(visible_left.saturating_sub(left)).ok()?;

    if let Some(clip) = context.clip {
        visible_rect = intersect_rect(visible_rect, clip)?;
        skip = skip.saturating_add(usize::from(visible_rect.x.saturating_sub(translated_x)));
    }

    if visible_rect.width == 0 || visible_rect.height == 0 {
        return None;
    }

    let visible: String = content
        .chars()
        .skip(skip)
        .take(usize::from(visible_rect.width))
        .collect();

    if visible.is_empty() {
        None
    } else {
        Some((visible_rect, visible))
    }
}

fn translate_rect(rect: Rect, scroll: ScrollOffset) -> Option<Rect> {
    Some(Rect::new(
        rect.x.checked_sub(scroll.x)?,
        rect.y.checked_sub(scroll.y)?,
        rect.width,
        rect.height,
    ))
}

fn combine_clip(existing: Option<Rect>, next: Option<Rect>) -> Option<Rect> {
    match (existing, next) {
        (Some(a), Some(b)) => intersect_rect(a, b),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

fn apply_clip(rect: Rect, clip: Option<Rect>) -> Option<Rect> {
    clip.map_or(Some(rect), |clip| intersect_rect(rect, clip))
}

fn intersect_rect(a: Rect, b: Rect) -> Option<Rect> {
    let x1 = a.x.max(b.x);
    let y1 = a.y.max(b.y);
    let x2 = a.x.saturating_add(a.width).min(b.x.saturating_add(b.width));
    let y2 =
        a.y.saturating_add(a.height)
            .min(b.y.saturating_add(b.height));

    if x1 >= x2 || y1 >= y2 {
        None
    } else {
        Some(Rect::new(x1, y1, x2 - x1, y2 - y1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Scene,
        layout::{Rect, resolve_layout},
        scene::{Padding, ScrollOffset, SizeConstraint},
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
    fn scroll_skips_horizontal_content() {
        let scene =
            Scene::<()>::scroll(1_u64, ScrollOffset::new(2, 0), Scene::text(2_u64, "abcdef"));
        let layout = resolve_layout(&scene, Rect::new(0, 0, 3, 1));
        let ops = render_ops(&layout);
        assert_eq!(
            ops,
            vec![RenderOp::DrawText {
                id: NodeId::new(2),
                rect: Rect::new(0, 0, 3, 1),
                content: "cde".into(),
                style: Style::PLAIN,
            }]
        );
    }

    #[test]
    fn scroll_shifts_content_vertically_within_viewport() {
        let scene = Scene::<()>::viewport(
            1_u64,
            Scene::scroll(
                2_u64,
                ScrollOffset::new(0, 1),
                Scene::column(
                    3_u64,
                    vec![
                        Scene::text(4_u64, "first"),
                        Scene::text(5_u64, "second"),
                        Scene::text(6_u64, "third"),
                    ],
                ),
            ),
        );
        let layout = resolve_layout(&scene, Rect::new(0, 0, 6, 2));
        let ops = render_ops(&layout);

        assert_eq!(
            ops,
            vec![
                RenderOp::DrawText {
                    id: NodeId::new(5),
                    rect: Rect::new(0, 0, 6, 1),
                    content: "second".into(),
                    style: Style::PLAIN,
                },
                RenderOp::DrawText {
                    id: NodeId::new(6),
                    rect: Rect::new(0, 1, 5, 1),
                    content: "third".into(),
                    style: Style::PLAIN,
                },
            ]
        );
    }

    #[test]
    fn stack_renders_children_in_order() {
        let scene = Scene::<()>::stack(
            1_u64,
            vec![
                Scene::text(2_u64, "base"),
                Scene::annotated(3_u64, "overlay", Scene::text(4_u64, "top")),
            ],
        );
        let layout = resolve_layout(&scene, Rect::new(0, 0, 10, 2));
        let ops = render_ops(&layout);

        assert_eq!(
            ops,
            vec![
                RenderOp::DrawText {
                    id: NodeId::new(2),
                    rect: Rect::new(0, 0, 4, 1),
                    content: "base".into(),
                    style: Style::PLAIN,
                },
                RenderOp::Annotate {
                    id: NodeId::new(3),
                    rect: Rect::new(0, 0, 10, 2),
                    label: "overlay".into(),
                },
                RenderOp::DrawText {
                    id: NodeId::new(4),
                    rect: Rect::new(0, 0, 3, 1),
                    content: "top".into(),
                    style: Style::PLAIN,
                },
            ]
        );
    }

    #[test]
    fn sized_is_structural_in_render_pass() {
        let scene = Scene::<()>::sized(
            1_u64,
            SizeConstraint::width(3),
            Scene::text(2_u64, "abcdef"),
        );
        let layout = resolve_layout(&scene, Rect::new(0, 0, 10, 1));
        let ops = render_ops(&layout);
        assert_eq!(
            ops,
            vec![RenderOp::DrawText {
                id: NodeId::new(2),
                rect: Rect::new(0, 0, 3, 1),
                content: "abc".into(),
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
