use crate::id::NodeId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Annotation {
    Label(String),
    SharedKey(String),
    FocusScope(String),
    PresenceSlot(String),
    RemoteCursor(NodeId),
}
