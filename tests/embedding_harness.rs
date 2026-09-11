// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

// Tier 0 embedding-contract validation harness.
//
// Exercises the Wallace ↔ Knopper embedding contract (IA-documents
// CONTRACTS/wallace-knopper-embedding.md v0.3) §1–§5 against the real
// runtime, using only the public embedding surface a host would use.
// See docs/embedding-validation-2026-08.md for the findings note.

use std::time::Instant;

use knopper::{
    Effect, Key, KeyEvent, MockBackend, NodeId, PureMachine, Rect, Runtime, RuntimeEvent, Scene,
};
use knopper::{FromGeometric, GA3, IntoGeometric};

// ---------------------------------------------------------------------------
// Host-side domain types — defined here, in "host crate" position, with no
// Knopper internals (contract §4: hosts define their own Msg).
// ---------------------------------------------------------------------------

/// Wallace-side analogue: events the host's domain produces.
#[derive(Debug, Clone, PartialEq, Eq)]
enum DomainEvent {
    RunStreamChunkReceived(String),
    ParticipantJoined(String),
}

/// Host-defined machine message. The host's adapter translates
/// `DomainEvent` → `TranscriptMsg`; Knopper never sees `DomainEvent`.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TranscriptMsg {
    Clear,
    ScrollDown,
    ScrollUp,
}

/// The derived projection a host computes from its session state and pushes
/// whole via `set_shared` (contract §3).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct TranscriptProjection {
    lines: Vec<String>,
}

impl IntoGeometric for TranscriptProjection {
    fn into_geometric(self) -> GA3 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.lines.hash(&mut hasher);
        let hash = hasher.finish();
        let coeffs: Vec<f64> = (0..8)
            .map(|i| match i {
                0 => (hash as f64) / (u64::MAX as f64),
                1 => self.lines.len() as f64,
                _ => 0.0,
            })
            .collect();
        GA3::from_coefficients(coeffs)
    }
}

impl FromGeometric for TranscriptProjection {
    fn from_geometric(mv: &GA3) -> Self {
        // Native value is cached by Behavior; the geometric form is only a
        // reactivity fingerprint, mirroring cliffy-core's String encoding.
        let _ = mv;
        Self::default()
    }
}

/// Participant-local UI state (contract §3: Model is never host-pushed).
type ScrollModel = usize;

/// Host application context (identity/theme analogue), pushed via
/// `set_context` when it changes.
#[derive(Debug, Clone, PartialEq, Eq)]
struct HostContext {
    participant: String,
}

/// The transcript machine, typed entirely with host-side types.
fn transcript_machine() -> impl knopper::Machine<
    Context = HostContext,
    Msg = TranscriptMsg,
    Model = ScrollModel,
    Shared = TranscriptProjection,
> {
    PureMachine::new(
        |_ctx: &HostContext| 0_usize,
        |model: &mut ScrollModel, msg: TranscriptMsg, _ctx: &HostContext| match msg {
            TranscriptMsg::Clear => {
                *model = 0;
                Effect::None
            }
            TranscriptMsg::ScrollDown => {
                *model += 1;
                Effect::None
            }
            TranscriptMsg::ScrollUp => {
                *model = model.saturating_sub(1);
                Effect::None
            }
        },
        |model: &ScrollModel, shared: &TranscriptProjection, ctx: &HostContext| {
            // Stable per-line ids: appends insert a new node without
            // disturbing existing ones (contract §5 append-mostly shape).
            let mut children: Vec<Scene<TranscriptMsg>> = shared
                .lines
                .iter()
                .enumerate()
                .map(|(i, line)| Scene::text(1_000 + i as u64, line.clone()))
                .collect();
            children.push(Scene::text(
                1_u64,
                format!(
                    "{} @ scroll {} — {} lines",
                    ctx.participant,
                    model,
                    shared.lines.len()
                ),
            ));
            Scene::column(2_u64, children)
        },
    )
}

const BOUNDS: Rect = Rect::new(0, 0, 80, 24);

fn host() -> Runtime<
    impl knopper::Machine<
        Context = HostContext,
        Msg = TranscriptMsg,
        Model = ScrollModel,
        Shared = TranscriptProjection,
    >,
> {
    Runtime::new(
        transcript_machine(),
        HostContext {
            participant: "elliot".into(),
        },
        TranscriptProjection::default(),
    )
}

// ---------------------------------------------------------------------------
// §1 — Embedding model: the Runtime surface is embedder-driven.
// ---------------------------------------------------------------------------

#[test]
fn s1_runtime_surface_is_embedder_driven() {
    let mut runtime = host();

    // Read surface.
    let _scene = runtime.scene();
    let _model: ScrollModel = runtime.model();
    let _shared: TranscriptProjection = runtime.shared();
    let _focus = runtime.focus();
    let _layout = runtime.layout(BOUNDS);
    let _ops = runtime.render_ops(BOUNDS);
    let _cursor = runtime.cursor(BOUNDS);
    let _patches = runtime.diff(BOUNDS);

    // Mutation surface.
    runtime.set_context(HostContext {
        participant: "renamed".into(),
    });
    runtime.set_shared(TranscriptProjection {
        lines: vec!["hello".into()],
    });
    runtime.dispatch(RuntimeEvent::Tick);
    runtime.send(TranscriptMsg::ScrollDown);

    // Commit surface.
    let mut backend = MockBackend::default();
    runtime.render_to_backend(&mut backend, BOUNDS).unwrap();
    runtime
        .render_to_backend_with_cursor(&mut backend, BOUNDS, Some((0, 0)))
        .unwrap();
    runtime
        .render_to_backend_auto_cursor(&mut backend, BOUNDS)
        .unwrap();
    runtime.invalidate_render_state();

    // Context and shared updates are visible through the projection.
    let ops = runtime.render_ops(BOUNDS);
    let header = ops
        .iter()
        .find_map(|op| match op {
            knopper::RenderOp::DrawText { id, content, .. } if *id == NodeId::new(1) => {
                Some(content.clone())
            }
            _ => None,
        })
        .expect("header row present");
    assert!(header.contains("renamed @ scroll 1 — 1 lines"));

    // Non-goal (contract §1): the library exposes no `run()` loop. The only
    // loops live in the demo binary (src/main.rs); the embedding surface
    // above is the whole turn cycle. Verified by inspection; asserted here
    // only insofar as this test itself is a hand-driven turn cycle.
}

// ---------------------------------------------------------------------------
// §2 — The drive loop: a host-driven turn cycle, no event-loop ownership.
// ---------------------------------------------------------------------------

#[test]
fn s2_host_driven_turn_cycle() {
    let mut runtime = host();
    let mut backend = MockBackend::default();
    let mut projection = TranscriptProjection::default();

    // Turn 1: a domain event arrives → translate → send.
    let domain = DomainEvent::RunStreamChunkReceived("first chunk".into());
    match domain {
        DomainEvent::RunStreamChunkReceived(chunk) => {
            projection.lines.push(chunk);
            runtime.set_shared(projection.clone()); // §3: push whole projection
        }
        DomainEvent::ParticipantJoined(_) => unreachable!(),
    }
    runtime.render_to_backend(&mut backend, BOUNDS).unwrap();
    assert!(
        backend.state().get(NodeId::new(1_000)).is_some(),
        "chunk row committed to backend"
    );

    // Turn 2: a key arrives → dispatch.
    runtime.dispatch(RuntimeEvent::Key(KeyEvent {
        key: Key::Tab,
        ctrl: false,
        alt: false,
        shift: false,
    }));

    // Turn 3: another chunk + a host-originated Msg.
    projection.lines.push("second chunk".into());
    runtime.set_shared(projection.clone());
    runtime.send(TranscriptMsg::ScrollDown);
    runtime.render_to_backend(&mut backend, BOUNDS).unwrap();

    assert_eq!(runtime.model(), 1, "host Msg drove participant-local Model");
    assert_eq!(runtime.shared().lines.len(), 2);
    assert!(backend.state().get(NodeId::new(1_001)).is_some());

    // Turn 4: a participant joins → the adapter appends a system line to the
    // projection and refreshes app context via set_context.
    let domain = DomainEvent::ParticipantJoined("ada".into());
    match domain {
        DomainEvent::ParticipantJoined(name) => {
            projection.lines.push(format!("{name} joined"));
            runtime.set_shared(projection.clone());
            runtime.set_context(HostContext {
                participant: "elliot (+1 guest)".into(),
            });
        }
        DomainEvent::RunStreamChunkReceived(_) => unreachable!(),
    }
    runtime.render_to_backend(&mut backend, BOUNDS).unwrap();
    assert!(backend.state().get(NodeId::new(1_002)).is_some());

    // Turn 5: host sends the remaining Msg variants (clear scroll, scroll up).
    runtime.send(TranscriptMsg::Clear);
    runtime.send(TranscriptMsg::ScrollUp); // saturates at 0
    assert_eq!(runtime.model(), 0);

    // The host reads cursor per turn (None here: transcript has no editor).
    let _ = runtime.cursor(BOUNDS);
}

// ---------------------------------------------------------------------------
// §3 — Projection-injection: set_shared reprojects, and composes with update.
// ---------------------------------------------------------------------------

#[test]
fn s3_set_shared_reprojects_and_composes_with_update() {
    let mut runtime = host();

    // Machine's own update path mutates Model.
    runtime.send(TranscriptMsg::ScrollDown);
    runtime.send(TranscriptMsg::ScrollDown);

    // Host whole-replaces Shared; both state lanes must flow into the
    // reprojected scene on the next sample.
    runtime.set_shared(TranscriptProjection {
        lines: vec!["a".into(), "b".into(), "c".into()],
    });

    let ops = runtime.render_ops(BOUNDS);
    let texts: Vec<String> = ops
        .iter()
        .filter_map(|op| match op {
            knopper::RenderOp::DrawText { content, .. } => Some(content.clone()),
            _ => None,
        })
        .collect();

    // Model lane (scroll = 2) survived the Shared replace…
    assert!(
        texts.iter().any(|t| t.contains("scroll 2")),
        "Model state composes with set_shared whole-replace: {texts:?}"
    );
    // …and the Shared lane reprojected all three pushed lines.
    for line in ["a", "b", "c"] {
        assert!(texts.iter().any(|t| t == line), "missing line {line}");
    }
}

/// §3 / §7 / roadmap-06 agreement: Model (focus, scroll) is participant-local.
/// Two runtimes hosting the same machine definition share nothing.
#[test]
fn s3_model_state_is_participant_local() {
    let mut runtime_a = host();
    let runtime_b = host();

    // Same Shared projection pushed to both participants…
    let projection = TranscriptProjection {
        lines: vec!["shared line".into()],
    };
    runtime_a.set_shared(projection.clone());
    runtime_b.set_shared(projection);

    // …then participant A scrolls and focuses a real node.
    runtime_a.send(TranscriptMsg::ScrollDown);
    runtime_a.dispatch(RuntimeEvent::Focus(NodeId::new(1_000)));

    // …but Model and focus do not leak across runtimes.
    assert_eq!(runtime_a.model(), 1);
    assert_eq!(runtime_b.model(), 0);
    assert_ne!(
        runtime_a
            .focus()
            .current()
            .and_then(knopper::FocusPath::current),
        runtime_b
            .focus()
            .current()
            .and_then(knopper::FocusPath::current)
    );
}

// ---------------------------------------------------------------------------
// §4 — Host-defined Msg works on PureMachine; Effect semantics are
// synchronous and fully drained by the runtime.
// ---------------------------------------------------------------------------

#[test]
fn s4_host_msg_and_effect_semantics() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum ChainMsg {
        Start,
        Middle,
        End,
    }

    let machine = PureMachine::new(
        |_ctx: &()| 0_usize,
        |model: &mut usize, msg: ChainMsg, _ctx: &()| match msg {
            ChainMsg::Start => {
                *model |= 1;
                // Emit self-dispatches synchronously (recursive).
                Effect::Emit(ChainMsg::Middle)
            }
            ChainMsg::Middle => {
                *model |= 2;
                // Batch applies in order; RequestFocus moves runtime focus.
                Effect::Batch(vec![
                    Effect::Emit(ChainMsg::End),
                    Effect::RequestFocus(NodeId::new(7)),
                ])
            }
            ChainMsg::End => {
                *model |= 4;
                Effect::None
            }
        },
        |_model: &usize, _shared: &(), _ctx: &()| Scene::text(7_u64, "chained"),
    );

    let mut runtime = Runtime::new(machine, (), ());
    runtime.send(ChainMsg::Start);

    // One send() drained the whole chain: no host draining loop exists or
    // is needed. Effects cannot reach the host — the enum is closed
    // (None / Emit / Batch / RequestFocus).
    assert_eq!(runtime.model(), 0b111);
    assert_eq!(
        runtime
            .focus()
            .current()
            .and_then(knopper::FocusPath::current),
        Some(NodeId::new(7)),
        "RequestFocus applied through the runtime dispatch path"
    );
}

// ---------------------------------------------------------------------------
// §5 — Streaming-append: patch count per append stays bounded (structural
// assertion, CI-stable), with an ignored wall-clock measurement for the
// findings note.
// ---------------------------------------------------------------------------

fn drive_appends<M>(
    runtime: &mut Runtime<M>,
    backend: &mut MockBackend,
    chunks: usize,
) -> Vec<usize>
where
    M: knopper::Machine<
            Context = HostContext,
            Msg = TranscriptMsg,
            Model = ScrollModel,
            Shared = TranscriptProjection,
        >,
{
    let mut projection = TranscriptProjection::default();
    let mut per_turn_commands = Vec::with_capacity(chunks);
    for i in 0..chunks {
        projection.lines.push(format!("chunk {i}"));
        runtime.set_shared(projection.clone());
        let before = backend.executed().len();
        runtime.render_to_backend(backend, BOUNDS).unwrap();
        per_turn_commands.push(backend.executed().len() - before);
    }
    per_turn_commands
}

#[test]
fn s5_append_patches_stay_bounded() {
    let mut runtime = host();
    let mut backend = MockBackend::default();

    let per_turn = drive_appends(&mut runtime, &mut backend, 1_000);

    // After the first turn (which inserts header + first row), each append
    // is a constant-size commit: one inserted row plus the header update
    // (its line count changes each append). A regression to O(n) backend
    // work per chunk fails this immediately.
    let steady = &per_turn[10..];
    assert!(
        steady.iter().all(|&n| n <= 4),
        "per-append backend commands should be bounded; got max {}",
        steady.iter().max().unwrap()
    );
    assert_eq!(runtime.shared().lines.len(), 1_000);
}

/// Wall-clock measurement for the findings note. Ignored by default:
/// run with `cargo test --test embedding_harness -- --ignored --nocapture`.
#[test]
#[ignore = "perf measurement; run explicitly"]
fn s5_append_wall_clock_measurement() {
    let mut runtime = host();
    let mut backend = MockBackend::default();
    let mut projection = TranscriptProjection::default();

    let total = 10_000;
    let mut samples = Vec::with_capacity(total);
    for i in 0..total {
        projection.lines.push(format!("chunk {i}"));
        let start = Instant::now();
        runtime.set_shared(projection.clone());
        runtime.render_to_backend(&mut backend, BOUNDS).unwrap();
        samples.push(start.elapsed());
    }

    let report = |window: &[std::time::Duration]| {
        let total: std::time::Duration = window.iter().sum();
        total.as_secs_f64() / window.len() as f64 * 1_000.0
    };

    println!("append cost (set_shared + render_to_backend), ms/append:");
    println!("  appends   1..=100 avg: {:8.3}", report(&samples[..100]));
    println!(
        "  appends 401..=500 avg: {:8.3}",
        report(&samples[400..500])
    );
    println!(
        "  appends 900..=1000 avg: {:8.3}",
        report(&samples[900..1000])
    );
    println!(
        "  appends 4k..=5k avg: {:8.3}",
        report(&samples[4_000..5_000])
    );
    println!("  appends 9k..=10k avg: {:8.3}", report(&samples[9_000..]));
    println!(
        "  max single append: {:8.3} ms",
        samples.iter().max().unwrap().as_secs_f64() * 1_000.0
    );
}
