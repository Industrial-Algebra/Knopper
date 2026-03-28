use crate::{annotation::Annotation, id::NodeId, style::Style};
use cliffy_core::{FromGeometric, GA3, IntoGeometric};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Role {
    Generic,
    Header,
    Footer,
    Sidebar,
    Editor,
    StatusLine,
    ListItem,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Interaction<Msg> {
    #[default]
    None,
    Activate(Msg),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeMeta {
    pub id: NodeId,
    pub role: Role,
    pub style: Style,
    pub annotations: Vec<Annotation>,
    pub focusable: bool,
}

impl NodeMeta {
    #[must_use]
    pub fn new(id: impl Into<NodeId>) -> Self {
        Self {
            id: id.into(),
            role: Role::Generic,
            style: Style::PLAIN,
            annotations: Vec::new(),
            focusable: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Padding {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

impl Padding {
    #[must_use]
    pub const fn all(amount: u16) -> Self {
        Self {
            top: amount,
            right: amount,
            bottom: amount,
            left: amount,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SizeConstraint {
    pub width: Option<u16>,
    pub height: Option<u16>,
}

impl SizeConstraint {
    #[must_use]
    pub const fn new(width: Option<u16>, height: Option<u16>) -> Self {
        Self { width, height }
    }

    #[must_use]
    pub const fn width(width: u16) -> Self {
        Self {
            width: Some(width),
            height: None,
        }
    }

    #[must_use]
    pub const fn height(height: u16) -> Self {
        Self {
            width: None,
            height: Some(height),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScrollOffset {
    pub x: u16,
    pub y: u16,
}

impl ScrollOffset {
    #[must_use]
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextNode<Msg> {
    pub meta: NodeMeta,
    pub content: String,
    pub interaction: Interaction<Msg>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scene<Msg> {
    Empty,
    Text(TextNode<Msg>),
    Row {
        meta: NodeMeta,
        children: Vec<Scene<Msg>>,
    },
    Column {
        meta: NodeMeta,
        children: Vec<Scene<Msg>>,
    },
    Stack {
        meta: NodeMeta,
        children: Vec<Scene<Msg>>,
    },
    Padding {
        meta: NodeMeta,
        padding: Padding,
        child: Box<Scene<Msg>>,
    },
    Sized {
        meta: NodeMeta,
        constraint: SizeConstraint,
        child: Box<Scene<Msg>>,
    },
    Viewport {
        meta: NodeMeta,
        child: Box<Scene<Msg>>,
    },
    Scroll {
        meta: NodeMeta,
        offset: ScrollOffset,
        child: Box<Scene<Msg>>,
    },
    Border {
        meta: NodeMeta,
        child: Box<Scene<Msg>>,
    },
    Annotated {
        meta: NodeMeta,
        label: String,
        child: Box<Scene<Msg>>,
    },
}

impl<Msg> Scene<Msg> {
    #[must_use]
    pub fn text(id: impl Into<NodeId>, content: impl Into<String>) -> Self {
        Self::Text(TextNode {
            meta: NodeMeta::new(id),
            content: content.into(),
            interaction: Interaction::None,
        })
    }

    #[must_use]
    pub fn row(id: impl Into<NodeId>, children: impl Into<Vec<Scene<Msg>>>) -> Self {
        Self::Row {
            meta: NodeMeta::new(id),
            children: children.into(),
        }
    }

    #[must_use]
    pub fn column(id: impl Into<NodeId>, children: impl Into<Vec<Scene<Msg>>>) -> Self {
        Self::Column {
            meta: NodeMeta::new(id),
            children: children.into(),
        }
    }

    #[must_use]
    pub fn stack(id: impl Into<NodeId>, children: impl Into<Vec<Scene<Msg>>>) -> Self {
        Self::Stack {
            meta: NodeMeta::new(id),
            children: children.into(),
        }
    }

    #[must_use]
    pub fn overlay(id: impl Into<NodeId>, children: impl Into<Vec<Scene<Msg>>>) -> Self {
        Self::stack(id, children)
    }

    #[must_use]
    pub fn padding(id: impl Into<NodeId>, padding: Padding, child: Scene<Msg>) -> Self {
        Self::Padding {
            meta: NodeMeta::new(id),
            padding,
            child: Box::new(child),
        }
    }

    #[must_use]
    pub fn sized(id: impl Into<NodeId>, constraint: SizeConstraint, child: Scene<Msg>) -> Self {
        Self::Sized {
            meta: NodeMeta::new(id),
            constraint,
            child: Box::new(child),
        }
    }

    #[must_use]
    pub fn viewport(id: impl Into<NodeId>, child: Scene<Msg>) -> Self {
        Self::Viewport {
            meta: NodeMeta::new(id),
            child: Box::new(child),
        }
    }

    #[must_use]
    pub fn scroll(id: impl Into<NodeId>, offset: ScrollOffset, child: Scene<Msg>) -> Self {
        Self::Scroll {
            meta: NodeMeta::new(id),
            offset,
            child: Box::new(child),
        }
    }

    #[must_use]
    pub fn border(id: impl Into<NodeId>, child: Scene<Msg>) -> Self {
        Self::Border {
            meta: NodeMeta::new(id),
            child: Box::new(child),
        }
    }

    #[must_use]
    pub fn annotated(id: impl Into<NodeId>, label: impl Into<String>, child: Scene<Msg>) -> Self {
        Self::Annotated {
            meta: NodeMeta::new(id),
            label: label.into(),
            child: Box::new(child),
        }
    }

    #[must_use]
    pub fn with_role(mut self, role: Role) -> Self {
        self.meta_mut().role = role;
        self
    }

    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        let meta = self.meta_mut();
        meta.style = meta.style.combine(style);
        self
    }

    #[must_use]
    pub fn with_annotation(mut self, annotation: Annotation) -> Self {
        self.meta_mut().annotations.push(annotation);
        self
    }

    #[must_use]
    pub fn focusable(mut self) -> Self {
        self.meta_mut().focusable = true;
        self
    }

    #[must_use]
    pub fn on_activate(mut self, msg: Msg) -> Self {
        if let Self::Text(node) = &mut self {
            node.interaction = Interaction::Activate(msg);
            node.meta.focusable = true;
        }
        self
    }

    #[must_use]
    pub fn meta(&self) -> Option<&NodeMeta> {
        match self {
            Self::Empty => None,
            Self::Text(node) => Some(&node.meta),
            Self::Row { meta, .. }
            | Self::Column { meta, .. }
            | Self::Stack { meta, .. }
            | Self::Padding { meta, .. }
            | Self::Sized { meta, .. }
            | Self::Viewport { meta, .. }
            | Self::Scroll { meta, .. }
            | Self::Border { meta, .. }
            | Self::Annotated { meta, .. } => Some(meta),
        }
    }

    fn meta_mut(&mut self) -> &mut NodeMeta {
        match self {
            Self::Empty => panic!("empty scene has no metadata"),
            Self::Text(node) => &mut node.meta,
            Self::Row { meta, .. }
            | Self::Column { meta, .. }
            | Self::Stack { meta, .. }
            | Self::Padding { meta, .. }
            | Self::Sized { meta, .. }
            | Self::Viewport { meta, .. }
            | Self::Scroll { meta, .. }
            | Self::Border { meta, .. }
            | Self::Annotated { meta, .. } => meta,
        }
    }

    #[must_use]
    pub fn map_msg<NextMsg>(self, f: &impl Fn(Msg) -> NextMsg) -> Scene<NextMsg> {
        match self {
            Self::Empty => Scene::Empty,
            Self::Text(node) => Scene::Text(TextNode {
                meta: node.meta,
                content: node.content,
                interaction: match node.interaction {
                    Interaction::None => Interaction::None,
                    Interaction::Activate(msg) => Interaction::Activate(f(msg)),
                },
            }),
            Self::Row { meta, children } => Scene::Row {
                meta,
                children: children.into_iter().map(|child| child.map_msg(f)).collect(),
            },
            Self::Column { meta, children } => Scene::Column {
                meta,
                children: children.into_iter().map(|child| child.map_msg(f)).collect(),
            },
            Self::Stack { meta, children } => Scene::Stack {
                meta,
                children: children.into_iter().map(|child| child.map_msg(f)).collect(),
            },
            Self::Padding {
                meta,
                padding,
                child,
            } => Scene::Padding {
                meta,
                padding,
                child: Box::new(child.map_msg(f)),
            },
            Self::Sized {
                meta,
                constraint,
                child,
            } => Scene::Sized {
                meta,
                constraint,
                child: Box::new(child.map_msg(f)),
            },
            Self::Viewport { meta, child } => Scene::Viewport {
                meta,
                child: Box::new(child.map_msg(f)),
            },
            Self::Scroll {
                meta,
                offset,
                child,
            } => Scene::Scroll {
                meta,
                offset,
                child: Box::new(child.map_msg(f)),
            },
            Self::Border { meta, child } => Scene::Border {
                meta,
                child: Box::new(child.map_msg(f)),
            },
            Self::Annotated { meta, label, child } => Scene::Annotated {
                meta,
                label,
                child: Box::new(child.map_msg(f)),
            },
        }
    }
}

impl<Msg> IntoGeometric for Scene<Msg> {
    fn into_geometric(self) -> GA3 {
        GA3::zero()
    }
}

impl<Msg> FromGeometric for Scene<Msg> {
    fn from_geometric(_mv: &GA3) -> Self {
        Self::Empty
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Color;

    #[test]
    fn maps_messages_across_scene_tree() {
        let scene = Scene::column(
            1_u64,
            vec![
                Scene::text(2_u64, "child-a").on_activate(1_u8),
                Scene::annotated(
                    3_u64,
                    "wrapper",
                    Scene::text(4_u64, "child-b").on_activate(2_u8),
                ),
            ],
        );

        let mapped = scene.map_msg(&|msg| match msg {
            1 => "increment",
            2 => "decrement",
            _ => "unknown",
        });

        let expected = Scene::column(
            1_u64,
            vec![
                Scene::text(2_u64, "child-a").on_activate("increment"),
                Scene::annotated(
                    3_u64,
                    "wrapper",
                    Scene::text(4_u64, "child-b").on_activate("decrement"),
                ),
            ],
        );

        assert_eq!(mapped, expected);
    }

    #[test]
    fn scene_metadata_accumulates_style_and_annotations() {
        let scene = Scene::<()>::text(10_u64, "hello")
            .with_role(Role::Header)
            .with_style(Style::PLAIN.fg(Color::Ansi(3)).bold())
            .with_annotation(Annotation::Label("title".into()))
            .focusable();

        let meta = scene.meta().expect("text nodes have metadata");
        assert_eq!(meta.role, Role::Header);
        assert_eq!(meta.style.fg, Some(Color::Ansi(3)));
        assert!(meta.style.emphasis.bold);
        assert_eq!(meta.annotations, vec![Annotation::Label("title".into())]);
        assert!(meta.focusable);
    }
}
