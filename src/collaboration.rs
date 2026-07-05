// Collaboration-ready semantic types for multi-participant Knopper sessions.
//
// These types are the canonical payload for a collaborative machine's `Shared`
// lane. They let downstream projects consume Knopper in functional
// multi-user form today: each participant runtime owns its own `Runtime`
// (with participant-local focus), and the replicated `Shared` state carries
// a `ParticipantRoster` describing who is present and what they are
// attending to. The scene projection derives presence overlays from it.
//
// Distribution (CRDT merge, network transport) is intentionally OUT of scope
// for these types. Downstream owns the convergence layer — it merges remote
// updates externally and pushes the merged `Shared` into Knopper via
// `Runtime::set_shared`. See docs/roadmap/06-collaboration-ready-contract.md.

use crate::NodeId;

/// Identity of a participant in a collaborative session.
///
/// Stable for the lifetime of a session. Used to key presence entries and to
/// distinguish the local participant from remote ones.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ParticipantId(String);

impl ParticipantId {
    /// Create a participant id from any string-like input.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// The participant's identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ParticipantId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for ParticipantId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// How present a participant appears in a derived overlay.
///
/// This is semantic tone, not presentation. Backends and projection code map
/// it to concrete styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PresenceTone {
    /// The local participant — the one this runtime is driving.
    Local,
    /// An active remote collaborator currently interacting.
    #[default]
    Collaborator,
    /// A passive observer (read-only or idle).
    Passive,
}

/// A single participant's published presence in a session.
///
/// Presence is participant-local published state: each participant owns and
/// publishes their own entry. The roster is the merged view of everyone's
/// published entries plus the local participant's own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Presence {
    /// Who this presence entry describes.
    pub id: ParticipantId,
    /// Human-readable display name.
    pub label: String,
    /// Semantic tone for overlay rendering.
    pub tone: PresenceTone,
    /// Semantic scene anchor this participant is currently attending to, if
    /// any. Stable `NodeId`s let overlays attach to the right surface across
    /// reprojection.
    pub anchor: Option<NodeId>,
}

impl Presence {
    /// Build a presence entry with required identity and label.
    #[must_use]
    pub fn new(id: ParticipantId, label: impl Into<String>, tone: PresenceTone) -> Self {
        Self {
            id,
            label: label.into(),
            tone,
            anchor: None,
        }
    }

    /// Attach a semantic anchor to this presence entry.
    #[must_use]
    pub fn with_anchor(mut self, anchor: NodeId) -> Self {
        self.anchor = Some(anchor);
        self
    }
}

/// The roster of participants in a collaborative session.
///
/// This is the intended canonical payload for a collaborative machine's
/// `Shared` lane. It distinguishes the local participant (driving this
/// runtime) from remote participants (whose presence is replicated in).
///
/// Downstream convergence layers (CRDT, server broadcast) merge remote
/// rosters into this shape and hand the result to `Runtime::set_shared`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParticipantRoster {
    local_id: Option<ParticipantId>,
    /// All known presence entries, including the local participant.
    participants: Vec<Presence>,
}

impl ParticipantRoster {
    /// Create an empty roster. Use `with_local` to declare the local
    /// participant.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create a roster rooted on a local participant.
    #[must_use]
    pub fn with_local(local: ParticipantId) -> Self {
        Self {
            local_id: Some(local.clone()),
            participants: vec![Presence {
                id: local,
                label: String::new(),
                tone: PresenceTone::Local,
                anchor: None,
            }],
        }
    }

    /// The local participant's id, if declared.
    #[must_use]
    pub fn local_id(&self) -> Option<&ParticipantId> {
        self.local_id.as_ref()
    }

    /// The local participant's presence entry, if declared.
    #[must_use]
    pub fn local(&self) -> Option<&Presence> {
        self.local_id
            .as_ref()
            .and_then(|id| self.participants.iter().find(|p| &p.id == id))
    }

    /// A mutable handle on the local participant's presence entry, if declared.
    pub fn local_mut(&mut self) -> Option<&mut Presence> {
        let id = self.local_id.clone()?;
        self.participants.iter_mut().find(|p| p.id == id)
    }

    /// Iterate over remote participants only (everyone except the local one).
    pub fn remotes(&self) -> impl Iterator<Item = &Presence> {
        let local = self.local_id.clone();
        self.participants
            .iter()
            .filter(move |p| local.as_ref().is_none_or(|id| &p.id != id))
    }

    /// Iterate over all participants (local first if declared, then remotes).
    pub fn all(&self) -> impl Iterator<Item = &Presence> {
        self.participants.iter()
    }

    /// Number of participants (local + remote).
    #[must_use]
    pub fn len(&self) -> usize {
        self.participants.len()
    }

    /// Whether the roster has no participants at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.participants.is_empty()
    }

    /// Upsert a presence entry. If an entry with the same id exists it is
    /// replaced; otherwise the entry is appended. The local participant's
    /// tone is forced to `Local` to keep the local/remote distinction honest.
    pub fn upsert(&mut self, presence: Presence) {
        let mut presence = presence;
        if self.local_id.as_ref().is_some_and(|id| id == &presence.id) {
            presence.tone = PresenceTone::Local;
        }
        if let Some(existing) = self.participants.iter_mut().find(|p| p.id == presence.id) {
            *existing = presence;
        } else {
            self.participants.push(presence);
        }
    }

    /// Remove a participant by id. Removing the local participant also clears
    /// the local id declaration.
    pub fn remove(&mut self, id: &ParticipantId) {
        self.participants.retain(|p| &p.id != id);
        if self.local_id.as_ref() == Some(id) {
            self.local_id = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn participant_id_round_trips_strings() {
        let id = ParticipantId::new("alice");
        assert_eq!(id.as_str(), "alice");
        assert_eq!(ParticipantId::from("bob"), ParticipantId::new("bob"));
        assert_eq!(ParticipantId::from(String::from("carol")).as_str(), "carol");
    }

    #[test]
    fn presence_new_builds_required_fields_and_optional_anchor() {
        let p = Presence::new(
            ParticipantId::new("alice"),
            "Alice",
            PresenceTone::Collaborator,
        );
        assert_eq!(p.id.as_str(), "alice");
        assert_eq!(p.label, "Alice");
        assert_eq!(p.tone, PresenceTone::Collaborator);
        assert_eq!(p.anchor, None);

        let anchored = p.with_anchor(NodeId::new(42));
        assert_eq!(anchored.anchor, Some(NodeId::new(42)));
    }

    #[test]
    fn roster_with_local_seeds_local_entry_and_distinguishes_remotes() {
        let mut roster = ParticipantRoster::with_local(ParticipantId::new("me"));
        roster.local_mut().unwrap().label = "Me".into();
        roster.upsert(Presence::new(
            ParticipantId::new("alice"),
            "Alice",
            PresenceTone::Collaborator,
        ));
        roster.upsert(Presence::new(
            ParticipantId::new("bob"),
            "Bob",
            PresenceTone::Passive,
        ));

        assert_eq!(roster.local_id().map(ParticipantId::as_str), Some("me"));
        assert_eq!(roster.local().unwrap().label, "Me");
        assert_eq!(roster.local().unwrap().tone, PresenceTone::Local);
        let remotes: Vec<_> = roster.remotes().map(|p| p.id.as_str()).collect();
        assert_eq!(remotes, vec!["alice", "bob"]);
        assert_eq!(roster.len(), 3);
        assert!(!roster.is_empty());
    }

    #[test]
    fn roster_upsert_replaces_existing_entry_and_keeps_local_tone_honest() {
        let mut roster = ParticipantRoster::with_local(ParticipantId::new("me"));
        // Try to mark the local participant as Passive — the roster should
        // refuse and force Local, so the local/remote distinction can't drift.
        roster.upsert(Presence::new(
            ParticipantId::new("me"),
            "Me",
            PresenceTone::Passive,
        ));
        assert_eq!(roster.local().unwrap().tone, PresenceTone::Local);

        // Upsert on a remote replaces in place.
        roster.upsert(Presence::new(
            ParticipantId::new("alice"),
            "Alice",
            PresenceTone::Collaborator,
        ));
        roster.upsert(Presence::new(
            ParticipantId::new("alice"),
            "Alice Q",
            PresenceTone::Passive,
        ));
        assert_eq!(roster.len(), 2);
        let alice = roster.remotes().next().unwrap();
        assert_eq!(alice.label, "Alice Q");
        assert_eq!(alice.tone, PresenceTone::Passive);
    }

    #[test]
    fn roster_remove_clears_entry_and_local_id_if_needed() {
        let mut roster = ParticipantRoster::with_local(ParticipantId::new("me"));
        roster.upsert(Presence::new(
            ParticipantId::new("alice"),
            "Alice",
            PresenceTone::Collaborator,
        ));

        roster.remove(&ParticipantId::new("alice"));
        assert_eq!(roster.len(), 1);
        assert!(roster.remotes().next().is_none());

        // Removing the local participant clears the local id declaration too.
        roster.remove(&ParticipantId::new("me"));
        assert_eq!(roster.local_id(), None);
        assert!(roster.local().is_none());
        assert!(roster.is_empty());
    }

    #[test]
    fn empty_roster_has_no_local_or_remotes() {
        let roster = ParticipantRoster::empty();
        assert!(roster.is_empty());
        assert_eq!(roster.len(), 0);
        assert!(roster.local_id().is_none());
        assert!(roster.local().is_none());
        assert_eq!(roster.remotes().count(), 0);
    }
}
