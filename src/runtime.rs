use crate::{
    Effect, FocusState, Machine, NodeId, RoutedEvent, RuntimeEvent, SceneBehavior,
    backend::{BackendCommand, TerminalBackend, backend_commands},
    diff::{PatchOp, diff_render_ops},
    layout::{LayoutNode, Rect, resolve_layout},
    render::{RenderOp, render_ops},
    renderer::Renderer,
    route_event,
};
use cliffy_core::{Behavior, FromGeometric, IntoGeometric, behavior};

pub struct Runtime<M>
where
    M: Machine,
{
    machine: M,
    model: Behavior<M::Model>,
    shared: Behavior<M::Shared>,
    scene: SceneBehavior<M::Msg>,
    focus: FocusState,
    ctx: M::Context,
    last_render_ops: Vec<RenderOp>,
}

impl<M> Runtime<M>
where
    M: Machine,
    M::Context: Clone,
    M::Model: Clone + IntoGeometric + FromGeometric + 'static,
    M::Shared: Clone + IntoGeometric + FromGeometric + 'static,
    M::Msg: Clone + 'static,
{
    #[must_use]
    pub fn new(machine: M, ctx: M::Context, shared_initial: M::Shared) -> Self {
        let model = behavior(machine.init(&ctx));
        let shared = behavior(shared_initial);
        let scene = machine.project(model.clone(), shared.clone(), &ctx);

        Self {
            machine,
            model,
            shared,
            scene,
            focus: FocusState::new(),
            ctx,
            last_render_ops: Vec::new(),
        }
    }

    #[must_use]
    pub fn scene(&self) -> &SceneBehavior<M::Msg> {
        &self.scene
    }

    #[must_use]
    pub fn model(&self) -> M::Model {
        self.model.sample()
    }

    #[must_use]
    pub fn shared(&self) -> M::Shared {
        self.shared.sample()
    }

    #[must_use]
    pub fn focus(&self) -> &FocusState {
        &self.focus
    }

    pub fn set_context(&mut self, ctx: M::Context) {
        self.ctx = ctx.clone();
        self.scene = self
            .machine
            .project(self.model.clone(), self.shared.clone(), &self.ctx);
    }

    pub fn set_shared(&self, shared: M::Shared) {
        self.shared.set(shared);
    }

    #[must_use]
    pub fn layout(&self, bounds: Rect) -> LayoutNode {
        resolve_layout(&self.scene.sample(), bounds)
    }

    #[must_use]
    pub fn render_ops(&self, bounds: Rect) -> Vec<RenderOp> {
        render_ops(&self.layout(bounds))
    }

    #[must_use]
    pub fn cursor(&self, bounds: Rect) -> Option<(u16, u16)> {
        let layout = self.layout(bounds);
        self.machine
            .cursor_position(
                &self.model.sample(),
                &self.shared.sample(),
                &self.ctx,
                &layout,
            )
            .map(|(x, y)| {
                (
                    x.min(bounds.width.saturating_sub(1)),
                    y.min(bounds.height.saturating_sub(1)),
                )
            })
    }

    #[must_use]
    pub fn diff(&self, bounds: Rect) -> Vec<PatchOp> {
        let next = self.render_ops(bounds);
        diff_render_ops(&self.last_render_ops, &next)
    }

    pub fn render<R: Renderer>(&mut self, renderer: &mut R, bounds: Rect) -> Result<(), R::Error> {
        let next = self.render_ops(bounds);
        let patches = diff_render_ops(&self.last_render_ops, &next);
        renderer.apply(&patches)?;
        self.last_render_ops = next;
        Ok(())
    }

    pub fn render_to_backend<B: TerminalBackend>(
        &mut self,
        backend: &mut B,
        bounds: Rect,
    ) -> Result<(), B::Error> {
        let next = self.render_ops(bounds);
        let patches = diff_render_ops(&self.last_render_ops, &next);
        let commands = backend_commands(&self.last_render_ops, &patches);
        backend.execute(&commands)?;
        self.last_render_ops = next;
        Ok(())
    }

    pub fn render_to_backend_with_cursor<B: TerminalBackend>(
        &mut self,
        backend: &mut B,
        bounds: Rect,
        cursor: Option<(u16, u16)>,
    ) -> Result<(), B::Error> {
        let next = self.render_ops(bounds);
        let patches = diff_render_ops(&self.last_render_ops, &next);
        let mut commands = backend_commands(&self.last_render_ops, &patches);
        commands.push(BackendCommand::SetCursor {
            id: NodeId::new(u64::MAX),
            position: cursor,
        });
        backend.execute(&commands)?;
        self.last_render_ops = next;
        Ok(())
    }

    pub fn render_to_backend_auto_cursor<B: TerminalBackend>(
        &mut self,
        backend: &mut B,
        bounds: Rect,
    ) -> Result<(), B::Error> {
        self.render_to_backend_with_cursor(backend, bounds, self.cursor(bounds))
    }

    pub fn dispatch(&mut self, event: RuntimeEvent) {
        match route_event(&self.scene.sample(), &mut self.focus, event) {
            RoutedEvent::Message(msg) => self.apply_message(msg),
            RoutedEvent::FocusChanged(_) | RoutedEvent::Ignored => {}
        }
    }

    pub fn send(&mut self, msg: M::Msg) {
        self.apply_message(msg);
    }

    fn apply_message(&mut self, msg: M::Msg) {
        let mut model = self.model.sample();
        let effect = self.machine.update(&mut model, msg, &self.ctx);
        self.model.set(model);
        self.apply_effect(effect);
    }

    fn apply_effect(&mut self, effect: Effect<M::Msg>) {
        match effect {
            Effect::None => {}
            Effect::Emit(msg) => self.apply_message(msg),
            Effect::Batch(effects) => {
                for effect in effects {
                    self.apply_effect(effect);
                }
            }
            Effect::RequestFocus(id) => self.dispatch(RuntimeEvent::Focus(id)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BackendEntry, MockRenderer, NodeId, PatchOp, PureMachine, Role, RuntimeEvent, Scene, Style,
        backend::{BackendCommand, MockBackend},
        layout::Rect,
        render::RenderOp,
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestContext {
        title: &'static str,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Msg {
        Increment,
        Double,
    }

    #[test]
    fn runtime_routes_activation_updates_model_and_reprojects_scene() {
        let machine = PureMachine::new(
            |_ctx: &TestContext| 0_i32,
            |model: &mut i32, msg: Msg, _ctx: &TestContext| {
                match msg {
                    Msg::Increment => *model += 1,
                    Msg::Double => *model *= 2,
                }
                Effect::None
            },
            |model: &i32, shared: &String, ctx: &TestContext| {
                Scene::text(2_u64, format!("{}:{}:{}", ctx.title, shared, model))
                    .with_role(Role::StatusLine)
                    .on_activate(Msg::Increment)
            },
        );

        let mut runtime = Runtime::new(
            machine,
            TestContext { title: "knopper" },
            "shared".to_string(),
        );
        runtime.dispatch(RuntimeEvent::Activate(NodeId::new(2)));

        assert_eq!(runtime.model(), 1);
        assert_eq!(
            runtime.scene().sample(),
            Scene::text(2_u64, "knopper:shared:1")
                .with_role(Role::StatusLine)
                .on_activate(Msg::Increment)
        );
    }

    #[test]
    fn runtime_applies_emitted_effects_recursively() {
        let machine = PureMachine::new(
            |_ctx: &TestContext| 1_i32,
            |model: &mut i32, msg: Msg, _ctx: &TestContext| match msg {
                Msg::Increment => {
                    *model += 1;
                    Effect::Emit(Msg::Double)
                }
                Msg::Double => {
                    *model *= 2;
                    Effect::None
                }
            },
            |model: &i32, _shared: &(), _ctx: &TestContext| {
                Scene::text(2_u64, model.to_string()).on_activate(Msg::Increment)
            },
        );

        let mut runtime = Runtime::new(machine, TestContext { title: "knopper" }, ());
        runtime.dispatch(RuntimeEvent::Activate(NodeId::new(2)));

        assert_eq!(runtime.model(), 4);
    }

    #[test]
    fn runtime_tracks_focus_and_enter_activation() {
        let machine = PureMachine::new(
            |_ctx: &TestContext| 0_i32,
            |model: &mut i32, msg: Msg, _ctx: &TestContext| {
                if let Msg::Increment = msg {
                    *model += 1;
                }
                Effect::RequestFocus(NodeId::new(2))
            },
            |model: &i32, _shared: &(), _ctx: &TestContext| {
                Scene::row(
                    1_u64,
                    vec![Scene::text(2_u64, format!("count:{model}")).on_activate(Msg::Increment)],
                )
            },
        );

        let mut runtime = Runtime::new(machine, TestContext { title: "knopper" }, ());
        runtime.dispatch(RuntimeEvent::Activate(NodeId::new(2)));
        assert_eq!(
            runtime
                .focus()
                .current()
                .and_then(crate::FocusPath::current),
            Some(NodeId::new(2))
        );

        runtime.dispatch(RuntimeEvent::Key(crate::KeyEvent {
            key: crate::Key::Enter,
            ctrl: false,
            alt: false,
            shift: false,
        }));

        assert_eq!(runtime.model(), 2);
    }

    #[test]
    fn runtime_shared_state_updates_reproject_scene() {
        let machine = PureMachine::new(
            |_ctx: &TestContext| 0_i32,
            |_model: &mut i32, _msg: Msg, _ctx: &TestContext| Effect::None,
            |model: &i32, shared: &String, _ctx: &TestContext| {
                Scene::text(1_u64, format!("{shared}:{model}"))
            },
        );

        let runtime = Runtime::new(machine, TestContext { title: "knopper" }, "a".to_string());
        assert_eq!(runtime.scene().sample(), Scene::text(1_u64, "a:0"));

        runtime.set_shared("b".to_string());
        assert_eq!(runtime.scene().sample(), Scene::text(1_u64, "b:0"));
    }

    #[test]
    fn runtime_can_produce_layout_and_render_ops() {
        let machine = PureMachine::new(
            |_ctx: &TestContext| 0_i32,
            |_model: &mut i32, _msg: Msg, _ctx: &TestContext| Effect::None,
            |_model: &i32, shared: &String, _ctx: &TestContext| {
                Scene::row(
                    1_u64,
                    vec![
                        Scene::text(2_u64, shared.clone()),
                        Scene::annotated(
                            3_u64,
                            "meta",
                            Scene::border(4_u64, Scene::text(5_u64, "ok")),
                        ),
                    ],
                )
            },
        );

        let runtime = Runtime::new(machine, TestContext { title: "knopper" }, "abc".to_string());
        let layout = runtime.layout(Rect::new(0, 0, 20, 2));
        let ops = runtime.render_ops(Rect::new(0, 0, 20, 2));

        assert_eq!(layout.id, NodeId::new(1));
        assert_eq!(
            ops,
            vec![
                RenderOp::DrawText {
                    id: NodeId::new(2),
                    rect: Rect::new(0, 0, 3, 1),
                    content: "abc".into(),
                    style: Style::PLAIN,
                },
                RenderOp::Annotate {
                    id: NodeId::new(3),
                    rect: Rect::new(3, 0, 4, 3),
                    label: "meta".into(),
                },
                RenderOp::DrawBorder {
                    id: NodeId::new(4),
                    rect: Rect::new(3, 0, 4, 3),
                    style: Style::PLAIN,
                },
                RenderOp::DrawText {
                    id: NodeId::new(5),
                    rect: Rect::new(4, 1, 2, 1),
                    content: "ok".into(),
                    style: Style::PLAIN,
                },
            ]
        );
    }

    #[test]
    fn runtime_diff_reports_insertions_against_empty_render_state() {
        let machine = PureMachine::new(
            |_ctx: &TestContext| 0_i32,
            |_model: &mut i32, _msg: Msg, _ctx: &TestContext| Effect::None,
            |_model: &i32, _shared: &(), _ctx: &TestContext| Scene::text(2_u64, "hello"),
        );

        let runtime = Runtime::new(machine, TestContext { title: "knopper" }, ());
        assert_eq!(
            runtime.diff(Rect::new(0, 0, 10, 1)),
            vec![PatchOp::Insert(RenderOp::DrawText {
                id: NodeId::new(2),
                rect: Rect::new(0, 0, 5, 1),
                content: "hello".into(),
                style: Style::PLAIN,
            })]
        );
    }

    #[test]
    fn runtime_render_updates_mock_renderer_and_tracks_previous_frame() {
        let machine = PureMachine::new(
            |_ctx: &TestContext| 0_i32,
            |model: &mut i32, msg: Msg, _ctx: &TestContext| {
                if let Msg::Increment = msg {
                    *model += 1;
                }
                Effect::None
            },
            |model: &i32, _shared: &(), _ctx: &TestContext| {
                Scene::text(2_u64, format!("count:{model}")).on_activate(Msg::Increment)
            },
        );

        let mut runtime = Runtime::new(machine, TestContext { title: "knopper" }, ());
        let mut renderer = MockRenderer::default();

        runtime
            .render(&mut renderer, Rect::new(0, 0, 20, 1))
            .expect("mock renderer should not fail");
        assert_eq!(
            renderer.applied(),
            &[PatchOp::Insert(RenderOp::DrawText {
                id: NodeId::new(2),
                rect: Rect::new(0, 0, 7, 1),
                content: "count:0".into(),
                style: Style::PLAIN,
            })]
        );

        runtime.dispatch(RuntimeEvent::Activate(NodeId::new(2)));
        runtime
            .render(&mut renderer, Rect::new(0, 0, 20, 1))
            .expect("mock renderer should not fail");

        assert_eq!(
            renderer.applied(),
            &[
                PatchOp::Insert(RenderOp::DrawText {
                    id: NodeId::new(2),
                    rect: Rect::new(0, 0, 7, 1),
                    content: "count:0".into(),
                    style: Style::PLAIN,
                }),
                PatchOp::Update(RenderOp::DrawText {
                    id: NodeId::new(2),
                    rect: Rect::new(0, 0, 7, 1),
                    content: "count:1".into(),
                    style: Style::PLAIN,
                }),
            ]
        );
    }

    #[test]
    fn runtime_can_translate_patches_into_backend_commands() {
        let machine = PureMachine::new(
            |_ctx: &TestContext| 0_i32,
            |_model: &mut i32, _msg: Msg, _ctx: &TestContext| Effect::None,
            |_model: &i32, _shared: &(), _ctx: &TestContext| {
                Scene::border(1_u64, Scene::text(2_u64, "hello"))
            },
        );

        let mut runtime = Runtime::new(machine, TestContext { title: "knopper" }, ());
        let mut backend = MockBackend::default();

        runtime
            .render_to_backend(&mut backend, Rect::new(0, 0, 10, 1))
            .expect("mock backend should not fail");

        assert_eq!(
            backend.executed(),
            &[
                BackendCommand::DrawBorder {
                    id: NodeId::new(1),
                    rect: Rect::new(0, 0, 10, 1),
                    style: Style::PLAIN,
                },
                BackendCommand::DrawText {
                    id: NodeId::new(2),
                    rect: Rect::new(1, 1, 5, 1),
                    content: "hello".into(),
                    style: Style::PLAIN,
                },
            ]
        );
    }

    #[test]
    fn runtime_backend_render_clears_previous_region_before_update() {
        let machine = PureMachine::new(
            |_ctx: &TestContext| 0_i32,
            |model: &mut i32, msg: Msg, _ctx: &TestContext| {
                if let Msg::Increment = msg {
                    *model += 1;
                }
                Effect::None
            },
            |model: &i32, _shared: &(), _ctx: &TestContext| {
                Scene::text(2_u64, format!("count:{model}")).on_activate(Msg::Increment)
            },
        );

        let mut runtime = Runtime::new(machine, TestContext { title: "knopper" }, ());
        let mut backend = MockBackend::default();

        runtime
            .render_to_backend(&mut backend, Rect::new(0, 0, 20, 1))
            .expect("mock backend should not fail");
        runtime.dispatch(RuntimeEvent::Activate(NodeId::new(2)));
        runtime
            .render_to_backend(&mut backend, Rect::new(0, 0, 20, 1))
            .expect("mock backend should not fail");

        assert_eq!(
            backend.executed(),
            &[
                BackendCommand::DrawText {
                    id: NodeId::new(2),
                    rect: Rect::new(0, 0, 7, 1),
                    content: "count:0".into(),
                    style: Style::PLAIN,
                },
                BackendCommand::ClearRect {
                    rect: Rect::new(0, 0, 7, 1),
                },
                BackendCommand::DrawText {
                    id: NodeId::new(2),
                    rect: Rect::new(0, 0, 7, 1),
                    content: "count:1".into(),
                    style: Style::PLAIN,
                },
            ]
        );
        assert_eq!(
            backend.state().get(NodeId::new(2)),
            Some(&BackendEntry::Text {
                rect: Rect::new(0, 0, 7, 1),
                content: "count:1".into(),
                style: Style::PLAIN,
            })
        );
    }

    #[test]
    fn runtime_can_send_cursor_commands_to_backend() {
        let machine = PureMachine::new(
            |_ctx: &TestContext| 0_i32,
            |_model: &mut i32, _msg: Msg, _ctx: &TestContext| Effect::None,
            |_model: &i32, _shared: &(), _ctx: &TestContext| Scene::text(2_u64, "hello"),
        );

        let mut runtime = Runtime::new(machine, TestContext { title: "knopper" }, ());
        let mut backend = MockBackend::default();
        runtime
            .render_to_backend_with_cursor(&mut backend, Rect::new(0, 0, 10, 1), Some((4, 0)))
            .expect("mock backend should not fail");

        assert_eq!(backend.state().cursor(), Some((4, 0)));
        assert_eq!(
            backend.executed().last(),
            Some(&BackendCommand::SetCursor {
                id: NodeId::new(u64::MAX),
                position: Some((4, 0)),
            })
        );
    }
}
