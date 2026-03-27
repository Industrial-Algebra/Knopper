use crate::{NodeId, Scene, Style};

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
            rect: Rect::new(bounds.x, bounds.y, text_width(&node.content), 1),
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
                    let child_rect =
                        Rect::new(x, bounds.y, size.width, bounds.height.max(size.height));
                    x = x.saturating_add(size.width);
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
                    let child_rect =
                        Rect::new(bounds.x, y, bounds.width.max(size.width), size.height);
                    y = y.saturating_add(size.height);
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
        Scene::Annotated { child, .. } => measure(child),
    }
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
