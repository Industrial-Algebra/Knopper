// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Reader DoD (identity-restoration plan, Unit 2; encoding contract §6):
//! a real consumer derives display-meaningful data **from the multivector
//! alone**, and its output changes when (and only when) the merge-relevant
//! state changes.

use cliffy_core::{GA3, IntoGeometric};
use knopper::annotation::Annotation;
use knopper::collaboration::{
    ParticipantId, ParticipantRoster, Presence, PresenceTone, ToneCensus, presence_annotation,
    tone_census,
};
use knopper::id::NodeId;

fn mixed_roster() -> ParticipantRoster {
    let mut roster = ParticipantRoster::with_local(ParticipantId::new("me"));
    roster.upsert(Presence::new(
        ParticipantId::new("alice"),
        "Alice",
        PresenceTone::Collaborator,
    ));
    roster.upsert(Presence::new(
        ParticipantId::new("bob"),
        "Bob",
        PresenceTone::Collaborator,
    ));
    roster.upsert(Presence::new(
        ParticipantId::new("carol"),
        "Carol",
        PresenceTone::Passive,
    ));
    roster
}

#[test]
fn census_reads_exact_counts_from_the_multivector() {
    let census = tone_census(&mixed_roster().into_geometric());
    assert_eq!(
        census,
        ToneCensus {
            participants: 4,
            local: 1,
            collaborator: 2,
            passive: 1
        }
    );
}

#[test]
fn annotation_is_display_meaningful() {
    let annotation = presence_annotation(&mixed_roster().into_geometric());
    assert_eq!(annotation, "you · 2 collaborators · 1 passive");
    // It feeds the existing presence-slot render surface directly.
    assert_eq!(
        Annotation::PresenceSlot(annotation.clone()),
        Annotation::PresenceSlot("you · 2 collaborators · 1 passive".to_string())
    );
}

#[test]
fn reader_works_from_geometry_alone_no_roster() {
    // Build the multivector by summing presence encodings directly — the
    // render path must not need the typed roster, only the geometry.
    let me = Presence::new(ParticipantId::new("me"), "Me", PresenceTone::Local);
    let a = Presence::new(ParticipantId::new("alice"), "A", PresenceTone::Collaborator);
    let b = Presence::new(ParticipantId::new("bob"), "B", PresenceTone::Collaborator);
    let c = Presence::new(ParticipantId::new("carol"), "C", PresenceTone::Passive);
    let mut mv = GA3::zero();
    for presence in [me, a, b, c] {
        mv = mv.add(&presence.into_geometric());
    }
    assert_eq!(
        presence_annotation(&mv),
        "you · 2 collaborators · 1 passive"
    );
}

#[test]
fn annotation_changes_when_tone_mix_changes() {
    let roster = mixed_roster();
    let before = presence_annotation(&roster.clone().into_geometric());

    let mut changed = roster;
    // carol goes passive -> collaborator: tone mix changes.
    changed.upsert(Presence::new(
        ParticipantId::new("carol"),
        "Carol",
        PresenceTone::Collaborator,
    ));
    let after = presence_annotation(&changed.into_geometric());
    assert_ne!(before, after);
    assert_eq!(after, "you · 3 collaborators");
}

#[test]
fn annotation_changes_when_count_changes() {
    let roster = mixed_roster();
    let before = presence_annotation(&roster.clone().into_geometric());

    let mut joined = roster;
    joined.upsert(Presence::new(
        ParticipantId::new("dave"),
        "Dave",
        PresenceTone::Passive,
    ));
    let after = presence_annotation(&joined.into_geometric());
    assert_ne!(before, after);
    assert_eq!(after, "you · 2 collaborators · 2 passives");
}

#[test]
fn relabel_moves_neither_geometry_nor_annotation() {
    // "Only when" (DoD): a change that does not touch merge semantics
    // (label churn) must not move the multivector or the derived display.
    let mut roster = mixed_roster();
    let before_mv = roster.clone().into_geometric();
    let before = presence_annotation(&before_mv);

    roster.upsert(Presence::new(
        ParticipantId::new("alice"),
        "Alice Renamed",
        PresenceTone::Collaborator,
    ));

    assert_eq!(roster.into_geometric(), before_mv, "relabel keeps geometry");
    assert_eq!(presence_annotation(&before_mv), before);
}

#[test]
fn anchor_change_moves_geometry_not_the_census() {
    // Anchoring is merge semantics (geometry moves) but not tone mix
    // (census stable) — the two readers disagree for the right reason.
    let mut roster = mixed_roster();
    let before_mv = roster.clone().into_geometric();

    roster.upsert(
        Presence::new(
            ParticipantId::new("alice"),
            "Alice",
            PresenceTone::Collaborator,
        )
        .with_anchor(NodeId::new(77)),
    );

    let after_mv = roster.into_geometric();
    assert_ne!(after_mv, before_mv, "anchor moves geometry");
    assert_eq!(tone_census(&after_mv), tone_census(&before_mv));
    assert_eq!(
        presence_annotation(&after_mv),
        presence_annotation(&before_mv)
    );
}

#[test]
fn empty_roster_reads_empty() {
    assert_eq!(
        presence_annotation(&ParticipantRoster::empty().into_geometric()),
        "empty"
    );
    assert_eq!(
        tone_census(&ParticipantRoster::empty().into_geometric()),
        ToneCensus::default()
    );
}
