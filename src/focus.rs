use crate::id::NodeId;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FocusPath(Vec<NodeId>);

impl FocusPath {
    #[must_use]
    pub fn root(root: NodeId) -> Self {
        Self(vec![root])
    }

    #[must_use]
    pub fn child(mut self, id: NodeId) -> Self {
        self.0.push(id);
        self
    }

    #[must_use]
    pub fn from_vec(path: Vec<NodeId>) -> Self {
        Self(path)
    }

    #[must_use]
    pub fn current(&self) -> Option<NodeId> {
        self.0.last().copied()
    }

    #[must_use]
    pub fn as_slice(&self) -> &[NodeId] {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FocusState {
    current: Option<FocusPath>,
}

impl FocusState {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, path: FocusPath) {
        self.current = Some(path);
    }

    #[must_use]
    pub fn current(&self) -> Option<&FocusPath> {
        self.current.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_path_tracks_current_node() {
        let path = FocusPath::root(NodeId::new(1)).child(NodeId::new(2));
        assert_eq!(path.current(), Some(NodeId::new(2)));
        assert_eq!(path.as_slice(), &[NodeId::new(1), NodeId::new(2)]);
    }
}
