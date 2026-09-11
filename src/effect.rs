// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use crate::id::NodeId;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Effect<Msg> {
    #[default]
    None,
    Emit(Msg),
    Batch(Vec<Effect<Msg>>),
    RequestFocus(NodeId),
}

impl<Msg> Effect<Msg> {
    #[must_use]
    pub fn batch(effects: impl IntoIterator<Item = Effect<Msg>>) -> Self {
        Self::Batch(effects.into_iter().collect())
    }
}
