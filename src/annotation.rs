// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use crate::id::NodeId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Annotation {
    Label(String),
    SharedKey(String),
    FocusScope(String),
    PresenceSlot(String),
    RemoteCursor(NodeId),
}
