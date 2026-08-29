// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};

use crate::{NodeId, RenderOp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchOp {
    Insert(RenderOp),
    Update(RenderOp),
    Remove(NodeId),
}

#[must_use]
pub fn diff_render_ops(previous: &[RenderOp], next: &[RenderOp]) -> Vec<PatchOp> {
    // O(n): index the previous frame by node id (first occurrence wins,
    // matching the linear-scan `find` semantics this replaces), then make
    // single passes for inserts/updates (in next order) and removals (in
    // previous order). Patch order is identical to the quadratic original.
    let mut previous_by_id: HashMap<NodeId, &RenderOp> = HashMap::with_capacity(previous.len());
    for op in previous {
        previous_by_id.entry(op_id(op)).or_insert(op);
    }

    let mut present: HashSet<NodeId> = HashSet::with_capacity(next.len());
    let mut patches = Vec::new();

    for op in next {
        let id = op_id(op);
        present.insert(id);
        match previous_by_id.get(&id) {
            None => patches.push(PatchOp::Insert(op.clone())),
            Some(&existing) if existing != op => patches.push(PatchOp::Update(op.clone())),
            Some(_) => {}
        }
    }

    for op in previous {
        let id = op_id(op);
        if !present.contains(&id) {
            patches.push(PatchOp::Remove(id));
        }
    }

    patches
}

fn op_id(op: &RenderOp) -> NodeId {
    match op {
        RenderOp::DrawText { id, .. }
        | RenderOp::DrawBorder { id, .. }
        | RenderOp::Annotate { id, .. }
        | RenderOp::SetCursor { id, .. } => *id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Style, layout::Rect};

    #[test]
    fn diff_detects_insert_update_and_remove() {
        let previous = vec![
            RenderOp::DrawText {
                id: NodeId::new(1),
                rect: Rect::new(0, 0, 3, 1),
                content: "old".into(),
                style: Style::PLAIN,
            },
            RenderOp::Annotate {
                id: NodeId::new(2),
                rect: Rect::new(0, 1, 3, 1),
                label: "gone".into(),
            },
        ];

        let next = vec![
            RenderOp::DrawText {
                id: NodeId::new(1),
                rect: Rect::new(0, 0, 3, 1),
                content: "new".into(),
                style: Style::PLAIN,
            },
            RenderOp::DrawText {
                id: NodeId::new(3),
                rect: Rect::new(0, 2, 2, 1),
                content: "++".into(),
                style: Style::PLAIN,
            },
        ];

        assert_eq!(
            diff_render_ops(&previous, &next),
            vec![
                PatchOp::Update(next[0].clone()),
                PatchOp::Insert(next[1].clone()),
                PatchOp::Remove(NodeId::new(2)),
            ]
        );
    }

    #[test]
    fn diff_ignores_unchanged_ops() {
        let ops = vec![RenderOp::DrawText {
            id: NodeId::new(1),
            rect: Rect::new(0, 0, 2, 1),
            content: "ok".into(),
            style: Style::PLAIN,
        }];

        assert!(diff_render_ops(&ops, &ops).is_empty());
    }
}
