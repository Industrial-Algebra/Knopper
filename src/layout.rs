use crate::{NodeId, Scene, Style, scene::Padding};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    #[must_use]
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    #[must_use]
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutNode {
    pub id: NodeId,
    pub rect: Rect,
    pub style: Style,
    pub kind: LayoutKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutKind {
    Empty,
    Text {
        content: String,
    },
    Row {
        children: Vec<LayoutNode>,
    },
    Column {
        children: Vec<LayoutNode>,
    },
    Padding {
        child: Box<LayoutNode>,
    },
    Viewport {
        child: Box<LayoutNode>,
    },
    Border {
        child: Box<LayoutNode>,
    },
    Annotated {
        label: String,
        child: Box<LayoutNode>,
    },
}

#[must_use]
pub fn resolve_layout<Msg>(scene: &Scene<Msg>, bounds: Rect) -> LayoutNode {
    resolve_with_inherited_style(scene, bounds, Style::PLAIN)
}

fn resolve_with_inherited_style<Msg>(
    scene: &Scene<Msg>,
    bounds: Rect,
    inherited: Style,
) -> LayoutNode {
    match scene {
        Scene::Empty => LayoutNode {
            id: NodeId::new(0),
            rect: Rect::new(bounds.x, bounds.y, 0, 0),
            style: inherited,
            kind: LayoutKind::Empty,
        },
        Scene::Text(node) => LayoutNode {
            id: node.meta.id,
            rect: Rect::new(
                bounds.x,
                bounds.y,
                text_width(&node.content).min(bounds.width),
                1,
            ),
            style: inherited.combine(node.meta.style),
            kind: LayoutKind::Text {
                content: node.content.clone(),
            },
        },
        Scene::Row { meta, children } => {
            let style = inherited.combine(meta.style);
            let child_sizes: Vec<Size> = children.iter().map(measure).collect();
            let mut x = bounds.x;
            let resolved_children = children
                .iter()
                .zip(child_sizes)
                .map(|(child, size)| {
                    let remaining = bounds.x.saturating_add(bounds.width).saturating_sub(x);
                    let child_rect = Rect::new(
                        x,
                        bounds.y,
                        size.width.min(remaining),
                        bounds.height.max(size.height),
                    );
                    x = x.saturating_add(child_rect.width);
                    resolve_with_inherited_style(child, child_rect, style)
                })
                .collect();

            LayoutNode {
                id: meta.id,
                rect: Rect::new(
                    bounds.x,
                    bounds.y,
                    x.saturating_sub(bounds.x),
                    bounds.height.max(1),
                ),
                style,
                kind: LayoutKind::Row {
                    children: resolved_children,
                },
            }
        }
        Scene::Column { meta, children } => {
            let style = inherited.combine(meta.style);
            let child_sizes: Vec<Size> = children.iter().map(measure).collect();
            let mut y = bounds.y;
            let resolved_children = children
                .iter()
                .zip(child_sizes)
                .map(|(child, size)| {
                    let remaining = bounds.y.saturating_add(bounds.height).saturating_sub(y);
                    let child_rect = Rect::new(
                        bounds.x,
                        y,
                        bounds.width.max(size.width),
                        size.height.min(remaining),
                    );
                    y = y.saturating_add(child_rect.height);
                    resolve_with_inherited_style(child, child_rect, style)
                })
                .collect();

            LayoutNode {
                id: meta.id,
                rect: Rect::new(
                    bounds.x,
                    bounds.y,
                    bounds.width.max(1),
                    y.saturating_sub(bounds.y),
                ),
                style,
                kind: LayoutKind::Column {
                    children: resolved_children,
                },
            }
        }
        Scene::Padding {
            meta,
            padding,
            child,
        } => {
            let style = inherited.combine(meta.style);
            let child_layout =
                resolve_with_inherited_style(child, inset_padding(bounds, *padding), style);
            LayoutNode {
                id: meta.id,
                rect: bounds,
                style,
                kind: LayoutKind::Padding {
                    child: Box::new(child_layout),
                },
            }
        }
        Scene::Viewport { meta, child } => {
            let style = inherited.combine(meta.style);
            let child_layout = resolve_with_inherited_style(child, bounds, style);
            LayoutNode {
                id: meta.id,
                rect: bounds,
                style,
                kind: LayoutKind::Viewport {
                    child: Box::new(child_layout),
                },
            }
        }
        Scene::Border { meta, child } => {
            let style = inherited.combine(meta.style);
            let inner = inset(bounds, 1);
            let child_layout = resolve_with_inherited_style(child, inner, style);
            LayoutNode {
                id: meta.id,
                rect: bounds,
                style,
                kind: LayoutKind::Border {
                    child: Box::new(child_layout),
                },
            }
        }
        Scene::Annotated { meta, label, child } => {
            let style = inherited.combine(meta.style);
            let child_layout = resolve_with_inherited_style(child, bounds, style);
            LayoutNode {
                id: meta.id,
                rect: bounds,
                style,
                kind: LayoutKind::Annotated {
                    label: label.clone(),
                    child: Box::new(child_layout),
                },
            }
        }
    }
}

#[must_use]
pub fn measure<Msg>(scene: &Scene<Msg>) -> Size {
    match scene {
        Scene::Empty => Size::new(0, 0),
        Scene::Text(node) => Size::new(text_width(&node.content), 1),
        Scene::Row { children, .. } => children.iter().fold(Size::default(), |acc, child| {
            let child_size = measure(child);
            Size::new(
                acc.width.saturating_add(child_size.width),
                acc.height.max(child_size.height),
            )
        }),
        Scene::Column { children, .. } => children.iter().fold(Size::default(), |acc, child| {
            let child_size = measure(child);
            Size::new(
                acc.width.max(child_size.width),
                acc.height.saturating_add(child_size.height),
            )
        }),
        Scene::Padding { padding, child, .. } => {
            let child_size = measure(child);
            Size::new(
                child_size
                    .width
                    .saturating_add(padding.left)
                    .saturating_add(padding.right),
                child_size
                    .height
                    .saturating_add(padding.top)
                    .saturating_add(padding.bottom),
            )
        }
        Scene::Viewport { child, .. } => measure(child),
        Scene::Border { child, .. } => {
            let child_size = measure(child);
            Size::new(
                child_size.width.saturating_add(2),
                child_size.height.saturating_add(2),
            )
        }
        Scene::Annotated { child, .. } => measure(child),
    }
}

fn inset(rect: Rect, amount: u16) -> Rect {
    let double = amount.saturating_mul(2);
    Rect::new(
        rect.x.saturating_add(amount),
        rect.y.saturating_add(amount),
        rect.width.saturating_sub(double).max(1),
        rect.height.saturating_sub(double).max(1),
    )
}

fn inset_padding(rect: Rect, padding: Padding) -> Rect {
    Rect::new(
        rect.x.saturating_add(padding.left),
        rect.y.saturating_add(padding.top),
        rect.width
            .saturating_sub(padding.left)
            .saturating_sub(padding.right)
            .max(1),
        rect.height
            .saturating_sub(padding.top)
            .saturating_sub(padding.bottom)
            .max(1),
    )
}

fn text_width(content: &str) -> u16 {
    u16::try_from(content.chars().count()).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, Scene};

    #[test]
    fn measures_rows_and_columns() {
        let row = Scene::<()>::row(
            1_u64,
            vec![Scene::text(2_u64, "abc"), Scene::text(3_u64, "de")],
        );
        let column = Scene::<()>::column(
            4_u64,
            vec![Scene::text(5_u64, "abc"), Scene::text(6_u64, "de")],
        );

        assert_eq!(measure(&row), Size::new(5, 1));
        assert_eq!(measure(&column), Size::new(3, 2));
    }

    #[test]
    fn padding_adds_space_to_measurement() {
        let padded = Scene::<()>::padding(1_u64, Padding::all(1), Scene::text(2_u64, "abc"));
        assert_eq!(measure(&padded), Size::new(5, 3));
    }

    #[test]
    fn border_adds_padding_to_measurement() {
        let bordered = Scene::<()>::border(1_u64, Scene::text(2_u64, "abc"));
        assert_eq!(measure(&bordered), Size::new(5, 3));
    }

    #[test]
    fn resolves_row_layout_positions_children_horizontally() {
        let scene = Scene::<()>::row(
            1_u64,
            vec![Scene::text(2_u64, "abc"), Scene::text(3_u64, "de")],
        );

        let layout = resolve_layout(&scene, Rect::new(0, 0, 20, 1));
        match layout.kind {
            LayoutKind::Row { children } => {
                assert_eq!(children[0].rect, Rect::new(0, 0, 3, 1));
                assert_eq!(children[1].rect, Rect::new(3, 0, 2, 1));
            }
            other => panic!("expected row layout, got {other:?}"),
        }
    }

    #[test]
    fn resolves_padding_layout_with_inset_child() {
        let scene = Scene::<()>::padding(1_u64, Padding::all(1), Scene::text(2_u64, "abc"));
        let layout = resolve_layout(&scene, Rect::new(0, 0, 5, 3));
        match layout.kind {
            LayoutKind::Padding { child } => {
                assert_eq!(child.rect, Rect::new(1, 1, 3, 1));
            }
            other => panic!("expected padding layout, got {other:?}"),
        }
    }

    #[test]
    fn resolves_border_layout_with_inset_child() {
        let scene = Scene::<()>::border(1_u64, Scene::text(2_u64, "abc"));
        let layout = resolve_layout(&scene, Rect::new(0, 0, 5, 3));
        match layout.kind {
            LayoutKind::Border { child } => {
                assert_eq!(child.rect, Rect::new(1, 1, 3, 1));
            }
            other => panic!("expected border layout, got {other:?}"),
        }
    }

    #[test]
    fn viewport_clips_text_width_to_bounds() {
        let scene = Scene::<()>::viewport(1_u64, Scene::text(2_u64, "abcdef"));
        let layout = resolve_layout(&scene, Rect::new(0, 0, 4, 1));
        match layout.kind {
            LayoutKind::Viewport { child } => {
                assert_eq!(child.rect, Rect::new(0, 0, 4, 1));
            }
            other => panic!("expected viewport layout, got {other:?}"),
        }
    }

    #[test]
    fn resolves_inherited_style_into_layout_nodes() {
        let scene = Scene::<()>::column(
            1_u64,
            vec![Scene::text(2_u64, "hello").with_style(Style::PLAIN.fg(Color::Ansi(5)).bold())],
        )
        .with_style(Style::PLAIN.bg(Color::Ansi(1)));

        let layout = resolve_layout(&scene, Rect::new(0, 0, 10, 3));
        match layout.kind {
            LayoutKind::Column { children } => {
                let child = &children[0];
                assert_eq!(child.style.fg, Some(Color::Ansi(5)));
                assert_eq!(child.style.bg, Some(Color::Ansi(1)));
                assert!(child.style.emphasis.bold);
            }
            other => panic!("expected column layout, got {other:?}"),
        }
    }
}
