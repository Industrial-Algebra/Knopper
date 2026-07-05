// Integration tests for the Knopper runtime pipeline.
//
// These exercise the full event -> route -> update -> effect -> reproject
// -> layout -> render -> diff -> backend-command path against the mock
// backend. They characterize the framework's interaction contracts so
// refactors do not silently regress end-to-end behavior.

use knopper::{
    BackendCommand, BackendEntry, Effect, FocusPath, FocusScopePolicy, KeyEvent, MockBackend,
    NodeId, PureMachine, Rect, Runtime, RuntimeEvent, Scene,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {
    IncFirst,
    IncSecond,
}

fn two_button_machine() -> impl knopper::Machine<Context = (), Msg = Msg, Model = i32, Shared = ()>
{
    PureMachine::new(
        |_ctx: &()| 0_i32,
        |model: &mut i32, msg: Msg, _ctx: &()| match msg {
            Msg::IncFirst => {
                *model += 1;
                Effect::None
            }
            Msg::IncSecond => {
                *model += 10;
                Effect::None
            }
        },
        |_model: &i32, _shared: &(), _ctx: &()| {
            Scene::column(
                1_u64,
                vec![
                    Scene::text(2_u64, "first").on_activate(Msg::IncFirst),
                    Scene::text(3_u64, "second").on_activate(Msg::IncSecond),
                ],
            )
        },
    )
}

#[test]
fn activation_routes_to_focused_node_and_updates_model() {
    let mut runtime = Runtime::new(two_button_machine(), (), ());
    let bounds = Rect::new(0, 0, 40, 10);

    // Focus the second button, then activate (Enter).
    runtime.dispatch(RuntimeEvent::Focus(NodeId::new(3)));
    runtime.dispatch(RuntimeEvent::Key(KeyEvent {
        key: knopper::Key::Enter,
        ctrl: false,
        alt: false,
        shift: false,
    }));

    assert_eq!(runtime.model(), 10);
    // Rendering produces backend entries for both texts.
    let mut backend = MockBackend::default();
    runtime.render_to_backend(&mut backend, bounds).unwrap();
    assert_eq!(backend.state().len(), 2);
    assert_matches_text(backend.state().get(NodeId::new(2)), "first");
    assert_matches_text(backend.state().get(NodeId::new(3)), "second");
}

#[test]
fn tab_traverses_focus_in_scene_order() {
    let mut runtime = Runtime::new(two_button_machine(), (), ());
    // No explicit focus set; Tab from None lands on the first focusable.
    runtime.dispatch(RuntimeEvent::Key(KeyEvent {
        key: knopper::Key::Tab,
        ctrl: false,
        alt: false,
        shift: false,
    }));
    assert_eq!(
        runtime.focus().current().and_then(FocusPath::current),
        Some(NodeId::new(2))
    );

    // Tab again moves to the second button.
    runtime.dispatch(RuntimeEvent::Key(KeyEvent {
        key: knopper::Key::Tab,
        ctrl: false,
        alt: false,
        shift: false,
    }));
    assert_eq!(
        runtime.focus().current().and_then(FocusPath::current),
        Some(NodeId::new(3))
    );

    // Shift-Tab moves back.
    runtime.dispatch(RuntimeEvent::Key(KeyEvent {
        key: knopper::Key::Tab,
        ctrl: false,
        alt: false,
        shift: true,
    }));
    assert_eq!(
        runtime.focus().current().and_then(FocusPath::current),
        Some(NodeId::new(2))
    );
}

#[test]
fn trap_scope_clamps_focus_within_scope_under_runtime_tab() {
    // A Trap scope around two buttons: Tab must clamp at the boundary and
    // never escape to the outside focusable. This is the end-to-end version
    // of the focus.rs unit test, proving the corrected Trap semantics hold
    // through the runtime dispatch path.
    let machine = PureMachine::new(
        |_ctx: &()| 0_i32,
        |m: &mut i32, msg: Msg, _ctx: &()| match msg {
            Msg::IncFirst => {
                *m += 1;
                Effect::None
            }
            Msg::IncSecond => {
                *m += 10;
                Effect::None
            }
        },
        |_m: &i32, _s: &(), _ctx: &()| {
            Scene::column(
                1_u64,
                vec![
                    Scene::text(2_u64, "outside").on_activate(Msg::IncFirst),
                    Scene::focus_scope_with_policy(
                        3_u64,
                        "trap",
                        FocusScopePolicy::Trap,
                        Scene::column(
                            4_u64,
                            vec![
                                Scene::text(5_u64, "in-a").on_activate(Msg::IncFirst),
                                Scene::text(6_u64, "in-b").on_activate(Msg::IncSecond),
                            ],
                        ),
                    ),
                ],
            )
        },
    );
    let mut runtime = Runtime::new(machine, (), ());

    // Land focus inside the trap on the first inner button.
    runtime.dispatch(RuntimeEvent::Focus(NodeId::new(5)));
    // Tab to the second inner button.
    runtime.dispatch(RuntimeEvent::Key(KeyEvent {
        key: knopper::Key::Tab,
        ctrl: false,
        alt: false,
        shift: false,
    }));
    assert_eq!(
        runtime.focus().current().and_then(FocusPath::current),
        Some(NodeId::new(6))
    );
    // Tab again must CLAMP (Trap), not escape to the outside node 2.
    runtime.dispatch(RuntimeEvent::Key(KeyEvent {
        key: knopper::Key::Tab,
        ctrl: false,
        alt: false,
        shift: false,
    }));
    assert_eq!(
        runtime.focus().current().and_then(FocusPath::current),
        Some(NodeId::new(6))
    );
}

#[test]
fn render_diffing_only_emits_changed_nodes() {
    // First render populates the backend; a no-op re-render should not
    // re-emit unchanged text. A model change that alters content should
    // emit a patch for the changed node only.
    let machine = PureMachine::new(
        |_ctx: &()| 0_i32,
        |m: &mut i32, msg: Msg, _ctx: &()| match msg {
            Msg::IncFirst => {
                *m += 1;
                Effect::None
            }
            Msg::IncSecond => {
                *m += 10;
                Effect::None
            }
        },
        |m: &i32, _s: &(), _ctx: &()| {
            Scene::column(
                1_u64,
                vec![
                    Scene::text(2_u64, format!("count: {}", m)),
                    Scene::text(3_u64, "static"),
                ],
            )
        },
    );
    let mut runtime = Runtime::new(machine, (), ());
    let bounds = Rect::new(0, 0, 40, 10);

    let mut backend = MockBackend::default();
    runtime.render_to_backend(&mut backend, bounds).unwrap();
    assert_eq!(backend.executed().len(), 2);
    assert_matches_text(backend.state().get(NodeId::new(2)), "count: 0");

    // Second render with no change: diff should be empty.
    backend = MockBackend::default();
    runtime.render_to_backend(&mut backend, bounds).unwrap();
    assert!(
        backend.executed().is_empty(),
        "no-op re-render should emit nothing"
    );

    // Update the model: only the dynamic node should re-emit.
    runtime.send(Msg::IncFirst);
    backend = MockBackend::default();
    runtime.render_to_backend(&mut backend, bounds).unwrap();
    assert!(
        backend
            .executed()
            .iter()
            .any(|c| matches!(c, BackendCommand::DrawText { id, .. } if id.get() == 2)),
        "expected a patch for the changed node"
    );
    assert!(
        !backend
            .executed()
            .iter()
            .any(|c| matches!(c, BackendCommand::DrawText { id, .. } if id.get() == 3)),
        "unchanged node must not be re-emitted"
    );
}

#[test]
fn request_focus_effect_moves_focus() {
    // A machine whose update emits Effect::RequestFocus should have the
    // runtime move focus to the requested node.
    let machine = PureMachine::new(
        |_ctx: &()| 0_i32,
        |m: &mut i32, msg: Msg, _ctx: &()| match msg {
            Msg::IncFirst => {
                *m += 1;
                Effect::RequestFocus(NodeId::new(3))
            }
            Msg::IncSecond => {
                *m += 10;
                Effect::None
            }
        },
        |_m: &i32, _s: &(), _ctx: &()| {
            Scene::column(
                1_u64,
                vec![
                    Scene::text(2_u64, "first").on_activate(Msg::IncFirst),
                    Scene::text(3_u64, "second").on_activate(Msg::IncSecond),
                ],
            )
        },
    );
    let mut runtime = Runtime::new(machine, (), ());

    runtime.dispatch(RuntimeEvent::Focus(NodeId::new(2)));
    runtime.dispatch(RuntimeEvent::Key(KeyEvent {
        key: knopper::Key::Enter,
        ctrl: false,
        alt: false,
        shift: false,
    }));
    // The IncFirst handler requested focus on node 3.
    assert_eq!(
        runtime.focus().current().and_then(FocusPath::current),
        Some(NodeId::new(3))
    );
}

fn assert_matches_text(entry: Option<&BackendEntry>, expected: &str) {
    match entry {
        Some(BackendEntry::Text { content, .. }) => {
            assert_eq!(content, expected);
        }
        other => panic!("expected Text entry matching {expected:?}, got {other:?}"),
    }
}
