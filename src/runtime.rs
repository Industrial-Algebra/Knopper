use crate::{Effect, FocusState, Machine, RoutedEvent, RuntimeEvent, SceneBehavior, route_event};
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

    pub fn set_shared(&self, shared: M::Shared) {
        self.shared.set(shared);
    }

    pub fn dispatch(&mut self, event: RuntimeEvent) {
        match route_event(&self.scene.sample(), &mut self.focus, event) {
            RoutedEvent::Message(msg) => self.apply_message(msg),
            RoutedEvent::FocusChanged(_) | RoutedEvent::Ignored => {}
        }
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
    use crate::{NodeId, PureMachine, Role, RuntimeEvent, Scene};

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
}
