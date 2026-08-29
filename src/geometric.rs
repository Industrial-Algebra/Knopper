// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Geometric encoding substrate — the implementation of the encoding
//! contract (`docs/design/geometric-encoding-contract.md`).
//!
//! Every `IntoGeometric` / `FromGeometric` impl in Knopper is built from
//! the pieces here: the [blade ledger](BLADES) (fixed role assignments
//! over GA3's 8 coefficients) and [`Digest`] (the canonical, versioned
//! structured-digest used by Class B encodings and aggregate composition).
//!
//! Two classes exist (contract §3):
//!
//! - **Class A (exact):** `from_geometric` inverts `into_geometric` on the
//!   nose — used whenever the encoded fields fit the ledger.
//! - **Class B (structured discriminant):** unbounded fields (strings,
//!   vecs, child states) pack length + [`Digest`] word pairs + exact
//!   scalars into named blades; distinct states yield distinct
//!   multivectors up to SHA-256 collision, and `from_geometric` returns
//!   [`Default`] (the typed cache is the reconstruction path, as in
//!   cliffy's own `String` handling).
//!
//! The collaboration lane (presence tones, roster sums) is *semantically
//! true* rather than Class A/B: tones are unit coefficients on their own
//! basis blades, so roster combination is multivector addition — see
//! `src/collaboration.rs` and contract §4.

use cliffy_core::GA3;
use sha2::{Digest as _ShaDigest, Sha256};

/// Number of blades in `GA3 = Multivector<3,0,0>` (Hadamard ordering).
pub const BLADES: usize = 8;

/// Blade index: `1` — identity / count / small primary scalar.
pub const SCALAR: usize = 0;
/// Blade index: `e1` — state axis A; collaboration: `Local` tone.
pub const E1: usize = 1;
/// Blade index: `e2` — state axis B; collaboration: `Collaborator` tone.
pub const E2: usize = 2;
/// Blade index: `e3` — state axis C / marker; collaboration: `Passive` tone.
pub const E3: usize = 3;
/// Blade index: `e12` — pairwise slot A (markers, digest word 1).
pub const E12: usize = 4;
/// Blade index: `e13` — pairwise slot B (markers, digest word 2).
pub const E13: usize = 5;
/// Blade index: `e23` — pairwise slot C (markers, digest word 3).
pub const E23: usize = 6;
/// Blade index: `e123` — pseudoscalar (aggregate / parity / digest word 4).
pub const E123: usize = 7;

/// Canonical structured digest (contract §3, Class B).
///
/// SHA-256, split into two 26-bit integer-valued `f64` words. Integer
/// words keep multivector **sums exact and order-independent** (critical
/// for the roster sum, contract §4): exact integers below 2^53 add
/// associatively in IEEE 754.
///
/// The `(0.0, 0.0)` pair is reserved as the "absent" marker for
/// `Option` payloads; real digests hit it with probability 2^-52.
///
/// **Version note:** the algorithm (SHA-256; words from hash bytes 0–4
/// and 4–8, `>> 6`) is pinned by the contract. Changing it breaks every
/// fingerprint derived from these encodings — it must not drift silently.
pub struct Digest;

impl Digest {
    /// Digest arbitrary bytes to a word pair.
    #[must_use]
    pub fn of_bytes(bytes: &[u8]) -> (f64, f64) {
        let hash = Sha256::digest(bytes);
        let lo = u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]) >> 6;
        let hi = u32::from_be_bytes([hash[4], hash[5], hash[6], hash[7]]) >> 6;
        (f64::from(lo), f64::from(hi))
    }

    /// Digest a multivector's canonical form (coefficient bit patterns) to
    /// a word pair. This is the composition operator for aggregates: a
    /// child's *encoding* is digested, so child encoding changes propagate
    /// automatically.
    #[must_use]
    pub fn of_multivector(mv: &GA3) -> (f64, f64) {
        let mut bytes = Vec::with_capacity(BLADES * 8);
        for i in 0..BLADES {
            bytes.extend_from_slice(&mv.get(i).to_bits().to_le_bytes());
        }
        Self::of_bytes(&bytes)
    }

    /// Digest a *sequence* of multivectors (child encodings of an
    /// aggregate). Children are fixed-width (8 coefficient bit patterns
    /// each), so plain concatenation is unambiguous: distinct child
    /// counts yield distinct digest-input lengths.
    #[must_use]
    pub fn of_children(children: &[GA3]) -> (f64, f64) {
        let mut bytes = Vec::new();
        for child in children {
            for i in 0..BLADES {
                bytes.extend_from_slice(&child.get(i).to_bits().to_le_bytes());
            }
        }
        Self::of_bytes(&bytes)
    }
}

/// Build a multivector from a filled coefficient array (all 8 blades).
#[must_use]
pub fn from_coeffs(coeffs: [f64; BLADES]) -> GA3 {
    GA3::from_slice(&coeffs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collaboration::{ParticipantId, ParticipantRoster, Presence, PresenceTone};
    use crate::id::NodeId;
    use crate::scene::Scene;
    use crate::standard::button::ButtonState;
    use crate::standard::input::InputState;
    use crate::standard::list::ListState;
    use crate::standard::list_detail::ListDetailState;
    use crate::standard::tabs::TabsState;
    use crate::standard::textarea::TextareaState;
    use crate::standard::toggle::ToggleState;
    use cliffy_core::{FromGeometric, IntoGeometric};

    // ------------------------------------------------------------------
    // Digest substrate
    // ------------------------------------------------------------------

    #[test]
    fn digest_is_deterministic_and_in_range() {
        let (a1, a2) = Digest::of_bytes(b"knopper");
        let (b1, b2) = Digest::of_bytes(b"knopper");
        assert_eq!((a1, a2), (b1, b2), "same bytes must digest identically");
        for w in [a1, a2] {
            assert!(
                w.fract() == 0.0 && (0.0..67_108_864.0).contains(&w),
                "26-bit word: {w}"
            );
        }
    }

    #[test]
    fn digest_distinguishes_bytes() {
        let a = Digest::of_bytes(b"value-a");
        let b = Digest::of_bytes(b"value-b");
        assert_ne!(a, b);
    }

    #[test]
    fn digest_of_multivector_is_stable() {
        let mv = GA3::scalar(7.0);
        assert_eq!(Digest::of_multivector(&mv), Digest::of_multivector(&mv));
        assert_ne!(
            Digest::of_multivector(&mv),
            Digest::of_multivector(&GA3::zero())
        );
    }

    #[test]
    fn digest_of_children_framing_is_unambiguous() {
        // Two children split one way must differ from one concatenated
        // child (framing prevents concatenation ambiguity).
        let x = GA3::scalar(1.0);
        let y = GA3::scalar(2.0);
        let pair = Digest::of_children(&[x, y]);
        let mut joined = GA3::zero();
        joined.set(0, 1.0);
        joined.set(1, 2.0);
        let single = Digest::of_children(&[joined]);
        assert_ne!(pair, single);
    }

    // ------------------------------------------------------------------
    // Class A encodings — exact round-trips
    // ------------------------------------------------------------------

    #[test]
    fn button_state_roundtrip_exact() {
        for activations in [0u64, 1, 42, 1_000_000] {
            let s = ButtonState { activations };
            let back = ButtonState::from_geometric(&s.into_geometric());
            assert_eq!(back.activations, activations);
        }
    }

    #[test]
    fn toggle_state_roundtrip_exact() {
        for checked in [false, true] {
            let s = ToggleState { checked };
            let back = ToggleState::from_geometric(&s.into_geometric());
            assert_eq!(back.checked, checked);
        }
    }

    #[test]
    fn list_state_roundtrip_exact() {
        let s = ListState {
            selected: 7,
            scroll: 320,
        };
        let back = ListState::from_geometric(&s.into_geometric());
        assert_eq!(back, s);
    }

    #[test]
    fn tabs_state_roundtrip_exact_including_none() {
        let none = TabsState {
            selected: 2,
            committed: None,
        };
        assert_eq!(TabsState::from_geometric(&none.into_geometric()), none);
        let some = TabsState {
            selected: 3,
            committed: Some(1),
        };
        assert_eq!(TabsState::from_geometric(&some.into_geometric()), some);
    }

    // ------------------------------------------------------------------
    // Class B encodings — change-soundness (distinct states, distinct
    // multivectors) + stability (same state, same multivector)
    // ------------------------------------------------------------------

    #[test]
    fn input_state_change_sound() {
        let base = InputState {
            value: "alpha".into(),
            cursor: 2,
            committed: None,
        };
        let mv_base = base.clone().into_geometric();
        assert_eq!(base.clone().into_geometric(), mv_base, "stable");

        let edit = InputState {
            value: "alphabet".into(),
            ..base.clone()
        };
        assert_ne!(edit.clone().into_geometric(), mv_base, "value change");

        let commit = InputState {
            committed: Some("alpha".into()),
            ..base.clone()
        };
        assert_ne!(commit.clone().into_geometric(), mv_base, "commit change");

        let none_vs_empty = InputState {
            committed: Some(String::new()),
            ..base.clone()
        };
        assert_ne!(
            none_vs_empty.into_geometric(),
            base.into_geometric(),
            "None vs Some(\"\") differ"
        );

        let move_cursor = InputState {
            cursor: 3,
            ..edit.clone()
        };
        assert_ne!(move_cursor.into_geometric(), edit.into_geometric());
    }

    #[test]
    fn textarea_state_change_sound() {
        let base = TextareaState {
            value: "line one\nline two".into(),
            cursor_row: 1,
            cursor_col: 4,
            preferred_col: None,
            committed: None,
        };
        let mv = base.clone().into_geometric();
        assert_eq!(base.clone().into_geometric(), mv);

        let pref = TextareaState {
            preferred_col: Some(4),
            ..base.clone()
        };
        assert_ne!(pref.into_geometric(), mv, "preferred col change");

        let commit = TextareaState {
            committed: Some("draft".into()),
            ..base
        };
        assert_ne!(commit.into_geometric(), mv, "commit change");
    }

    #[test]
    fn list_detail_composes_through_child_encoding() {
        let base = ListDetailState {
            list: ListState {
                selected: 1,
                scroll: 0,
            },
            committed: None,
        };
        let mv = base.clone().into_geometric();
        assert_eq!(base.clone().into_geometric(), mv);
        let moved = ListDetailState {
            list: ListState {
                selected: 2,
                scroll: 0,
            },
            committed: None,
        };
        assert_ne!(moved.into_geometric(), mv, "child list change");
    }

    #[test]
    fn command_palette_change_sound() {
        use crate::standard::command_palette::CommandPaletteState;
        let base = CommandPaletteState::default();
        let mv = base.clone().into_geometric();
        assert_eq!(base.clone().into_geometric(), mv);
        let typed = CommandPaletteState {
            input: InputState {
                value: "se".into(),
                ..base.input.clone()
            },
            ..base.clone()
        };
        assert_ne!(typed.into_geometric(), mv, "child input change");
        let closed = CommandPaletteState {
            open: false,
            ..base
        };
        assert_ne!(closed.into_geometric(), mv, "open toggle");
    }

    #[test]
    fn scene_fingerprint_change_sound() {
        let base: Scene<()> = Scene::column(
            NodeId::new(1),
            vec![
                Scene::text(NodeId::new(2), "hello"),
                Scene::text(NodeId::new(3), "world"),
            ],
        );
        let mv = base.clone().into_geometric();
        assert_eq!(base.clone().into_geometric(), mv, "stable");
        assert_eq!(
            mv.get(SCALAR),
            3.0,
            "node count: column + two text children"
        );

        let edited: Scene<()> = Scene::column(
            NodeId::new(1),
            vec![
                Scene::text(NodeId::new(2), "hello!"),
                Scene::text(NodeId::new(3), "world"),
            ],
        );
        assert_ne!(edited.into_geometric(), mv, "text change");

        let restructured: Scene<()> = Scene::row(
            NodeId::new(1),
            vec![
                Scene::text(NodeId::new(2), "hello"),
                Scene::text(NodeId::new(3), "world"),
            ],
        );
        assert_ne!(restructured.into_geometric(), mv, "kind change");
    }

    // ------------------------------------------------------------------
    // Collaboration lane — semantically true (contract §4)
    // ------------------------------------------------------------------

    #[test]
    fn tones_occupy_disjoint_blades() {
        let local = Presence::new(ParticipantId::new("a"), "A", PresenceTone::Local);
        let collab = Presence::new(ParticipantId::new("b"), "B", PresenceTone::Collaborator);
        let passive = Presence::new(ParticipantId::new("c"), "C", PresenceTone::Passive);
        let mv_local = local.into_geometric();
        let mv_collab = collab.into_geometric();
        let mv_passive = passive.into_geometric();
        assert_eq!(mv_local.get(E1), 1.0);
        assert_eq!(mv_local.get(E2), 0.0);
        assert_eq!(mv_collab.get(E2), 1.0);
        assert_eq!(mv_collab.get(E1), 0.0);
        assert_eq!(mv_passive.get(E3), 1.0);
        assert_eq!(mv_passive.get(E1), 0.0);
        assert_eq!(
            mv_local.get(SCALAR),
            1.0,
            "each presence counts one participant"
        );
    }

    #[test]
    fn roster_is_sum_of_presences_and_commutative() {
        let mut roster = ParticipantRoster::with_local(ParticipantId::new("me"));
        roster.upsert(Presence::new(
            ParticipantId::new("b"),
            "B",
            PresenceTone::Collaborator,
        ));
        roster.upsert(Presence::new(
            ParticipantId::new("c"),
            "C",
            PresenceTone::Collaborator,
        ));
        roster.upsert(Presence::new(
            ParticipantId::new("d"),
            "D",
            PresenceTone::Passive,
        ));

        let mv = roster.clone().into_geometric();
        assert_eq!(mv.get(SCALAR), 4.0, "participant count in scalar");
        assert_eq!(mv.get(E1), 1.0, "one local");
        assert_eq!(mv.get(E2), 2.0, "two collaborators");
        assert_eq!(mv.get(E3), 1.0, "one passive");

        // Commutativity: reordering participants must not change the sum.
        let mut reordered = ParticipantRoster::with_local(ParticipantId::new("me"));
        reordered.upsert(Presence::new(
            ParticipantId::new("d"),
            "D",
            PresenceTone::Passive,
        ));
        reordered.upsert(Presence::new(
            ParticipantId::new("c"),
            "C",
            PresenceTone::Collaborator,
        ));
        reordered.upsert(Presence::new(
            ParticipantId::new("b"),
            "B",
            PresenceTone::Collaborator,
        ));
        assert_eq!(
            reordered.into_geometric(),
            mv,
            "roster sum is order-independent"
        );
    }

    #[test]
    fn presence_encoding_excludes_labels_by_design() {
        // Labels are presentation, not merge semantics (contract §4):
        // relabeling must not move the geometry.
        let a = Presence::new(ParticipantId::new("x"), "Original", PresenceTone::Local);
        let b = Presence::new(ParticipantId::new("x"), "Renamed", PresenceTone::Local);
        assert_eq!(a.clone().into_geometric(), b.into_geometric());

        // But tone or anchor changes must.
        let anchored = a.clone().with_anchor(NodeId::new(99));
        assert_ne!(a.into_geometric(), anchored.into_geometric());
    }
}
