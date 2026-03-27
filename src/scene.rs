use crate::id::NodeId;
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
pub struct TextNode<Msg> {
    pub id: NodeId,
    pub role: Role,
    pub content: String,
    pub interaction: Interaction<Msg>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scene<Msg> {
    Empty,
    Text(TextNode<Msg>),
    Row {
        id: NodeId,
        role: Role,
        children: Vec<Scene<Msg>>,
    },
    Column {
        id: NodeId,
        role: Role,
        children: Vec<Scene<Msg>>,
    },
    Annotated {
        id: NodeId,
        role: Role,
        label: String,
        child: Box<Scene<Msg>>,
    },
}

impl<Msg> Scene<Msg> {
    #[must_use]
    pub fn text(id: impl Into<NodeId>, content: impl Into<String>) -> Self {
        Self::Text(TextNode {
            id: id.into(),
            role: Role::Generic,
            content: content.into(),
            interaction: Interaction::None,
        })
    }

    #[must_use]
    pub fn row(id: impl Into<NodeId>, children: impl Into<Vec<Scene<Msg>>>) -> Self {
        Self::Row {
            id: id.into(),
            role: Role::Generic,
            children: children.into(),
        }
    }

    #[must_use]
    pub fn column(id: impl Into<NodeId>, children: impl Into<Vec<Scene<Msg>>>) -> Self {
        Self::Column {
            id: id.into(),
            role: Role::Generic,
            children: children.into(),
        }
    }

    #[must_use]
    pub fn annotated(id: impl Into<NodeId>, label: impl Into<String>, child: Scene<Msg>) -> Self {
        Self::Annotated {
            id: id.into(),
            role: Role::Generic,
            label: label.into(),
            child: Box::new(child),
        }
    }

    #[must_use]
    pub fn with_role(mut self, role: Role) -> Self {
        match &mut self {
            Self::Empty => {}
            Self::Text(node) => node.role = role,
            Self::Row { role: current, .. }
            | Self::Column { role: current, .. }
            | Self::Annotated { role: current, .. } => *current = role,
        }
        self
    }

    #[must_use]
    pub fn on_activate(mut self, msg: Msg) -> Self {
        if let Self::Text(node) = &mut self {
            node.interaction = Interaction::Activate(msg);
        }
        self
    }

    #[must_use]
    pub fn map_msg<NextMsg>(self, f: &impl Fn(Msg) -> NextMsg) -> Scene<NextMsg> {
        match self {
            Self::Empty => Scene::Empty,
            Self::Text(node) => Scene::Text(TextNode {
                id: node.id,
                role: node.role,
                content: node.content,
                interaction: match node.interaction {
                    Interaction::None => Interaction::None,
                    Interaction::Activate(msg) => Interaction::Activate(f(msg)),
                },
            }),
            Self::Row { id, role, children } => Scene::Row {
                id,
                role,
                children: children.into_iter().map(|child| child.map_msg(f)).collect(),
            },
            Self::Column { id, role, children } => Scene::Column {
                id,
                role,
                children: children.into_iter().map(|child| child.map_msg(f)).collect(),
            },
            Self::Annotated {
                id,
                role,
                label,
                child,
            } => Scene::Annotated {
                id,
                role,
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
}
