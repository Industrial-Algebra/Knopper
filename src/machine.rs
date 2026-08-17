// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use crate::{Effect, LayoutNode, Scene};
use cliffy_core::{Behavior, FromGeometric, IntoGeometric, behavior};
use core::marker::PhantomData;

pub type SceneBehavior<Msg> = Behavior<Scene<Msg>>;

pub trait Machine {
    type Context;
    type Msg;
    type Model;
    type Shared;

    fn init(&self, ctx: &Self::Context) -> Self::Model;

    fn update(
        &self,
        model: &mut Self::Model,
        msg: Self::Msg,
        ctx: &Self::Context,
    ) -> Effect<Self::Msg>;

    fn project(
        &self,
        model: Behavior<Self::Model>,
        shared: Behavior<Self::Shared>,
        ctx: &Self::Context,
    ) -> SceneBehavior<Self::Msg>;

    fn project_once(
        &self,
        model: &Self::Model,
        shared: &Self::Shared,
        ctx: &Self::Context,
    ) -> Scene<Self::Msg>
    where
        Self::Msg: Clone + 'static,
        Self::Model: Clone + IntoGeometric + FromGeometric + 'static,
        Self::Shared: Clone + IntoGeometric + FromGeometric + 'static,
    {
        self.project(behavior(model.clone()), behavior(shared.clone()), ctx)
            .sample()
    }

    fn cursor_position(
        &self,
        _model: &Self::Model,
        _shared: &Self::Shared,
        _ctx: &Self::Context,
        _layout: &LayoutNode,
    ) -> Option<(u16, u16)> {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PureMachine<Context, Msg, Model, Shared, Init, Update, View> {
    init: Init,
    update: Update,
    view: View,
    _phantom: PhantomData<(Context, Msg, Model, Shared)>,
}

impl<Context, Msg, Model, Shared, Init, Update, View>
    PureMachine<Context, Msg, Model, Shared, Init, Update, View>
{
    #[must_use]
    pub fn new(init: Init, update: Update, view: View) -> Self {
        Self {
            init,
            update,
            view,
            _phantom: PhantomData,
        }
    }
}

impl<Context, Msg, Model, Shared, Init, Update, View> Machine
    for PureMachine<Context, Msg, Model, Shared, Init, Update, View>
where
    Context: Clone + 'static,
    Msg: Clone + 'static,
    Model: Clone + IntoGeometric + FromGeometric + 'static,
    Shared: Clone + IntoGeometric + FromGeometric + 'static,
    Init: Fn(&Context) -> Model,
    Update: Fn(&mut Model, Msg, &Context) -> Effect<Msg>,
    View: Fn(&Model, &Shared, &Context) -> Scene<Msg> + Clone + 'static,
{
    type Context = Context;
    type Msg = Msg;
    type Model = Model;
    type Shared = Shared;

    fn init(&self, ctx: &Self::Context) -> Self::Model {
        (self.init)(ctx)
    }

    fn update(
        &self,
        model: &mut Self::Model,
        msg: Self::Msg,
        ctx: &Self::Context,
    ) -> Effect<Self::Msg> {
        (self.update)(model, msg, ctx)
    }

    fn project(
        &self,
        model: Behavior<Self::Model>,
        shared: Behavior<Self::Shared>,
        ctx: &Self::Context,
    ) -> SceneBehavior<Self::Msg> {
        let view = self.view.clone();
        let initial = view(&model.sample(), &shared.sample(), ctx);
        let scene = behavior(initial);

        let scene_from_model = scene.clone();
        let shared_for_model = shared.clone();
        let ctx_for_model = ctx.clone();
        let view_for_model = view.clone();
        model.subscribe(move |next_model| {
            let next_scene = view_for_model(next_model, &shared_for_model.sample(), &ctx_for_model);
            scene_from_model.set(next_scene);
        });

        let scene_from_shared = scene.clone();
        let model_for_shared = model.clone();
        let ctx_for_shared = ctx.clone();
        shared.subscribe(move |next_shared| {
            let next_scene = view(&model_for_shared.sample(), next_shared, &ctx_for_shared);
            scene_from_shared.set(next_scene);
        });

        scene
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NodeId, Role};

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestContext {
        title: &'static str,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum CounterMsg {
        Increment,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum ParentMsg {
        Child(CounterMsg),
    }

    #[test]
    fn pure_machine_projects_a_scene_from_model_and_shared_state() {
        let machine = PureMachine::new(
            |_ctx: &TestContext| 0_i32,
            |model: &mut i32, msg: CounterMsg, _ctx: &TestContext| {
                match msg {
                    CounterMsg::Increment => *model += 1,
                }
                Effect::None
            },
            |model: &i32, shared: &String, ctx: &TestContext| {
                Scene::text(1_u64, format!("{}:{}:{}", ctx.title, shared, model))
                    .with_role(Role::StatusLine)
                    .on_activate(CounterMsg::Increment)
            },
        );

        let ctx = TestContext { title: "knopper" };
        let model = behavior(machine.init(&ctx));
        let shared = behavior("shared".to_string());

        let scene = machine.project(model.clone(), shared.clone(), &ctx);
        assert_eq!(
            scene.sample(),
            Scene::text(1_u64, "knopper:shared:0")
                .with_role(Role::StatusLine)
                .on_activate(CounterMsg::Increment)
        );

        model.set(2);
        assert_eq!(
            scene.sample(),
            Scene::text(1_u64, "knopper:shared:2")
                .with_role(Role::StatusLine)
                .on_activate(CounterMsg::Increment)
        );

        shared.set("remote".to_string());
        assert_eq!(
            scene.sample(),
            Scene::text(1_u64, "knopper:remote:2")
                .with_role(Role::StatusLine)
                .on_activate(CounterMsg::Increment)
        );
    }

    #[test]
    fn child_scene_messages_can_be_lifted_into_parent_messages() {
        let child_scene = Scene::text(NodeId::new(7), "counter").on_activate(CounterMsg::Increment);
        let parent_scene = Scene::column(
            1_u64,
            vec![
                child_scene.map_msg(&ParentMsg::Child),
                Scene::text(2_u64, "status"),
            ],
        );

        let expected = Scene::column(
            1_u64,
            vec![
                Scene::text(NodeId::new(7), "counter")
                    .on_activate(ParentMsg::Child(CounterMsg::Increment)),
                Scene::text(2_u64, "status"),
            ],
        );

        assert_eq!(parent_scene, expected);
    }

    #[test]
    fn update_can_emit_structured_effects() {
        let machine = PureMachine::new(
            |_ctx: &TestContext| 0_i32,
            |model: &mut i32, msg: CounterMsg, _ctx: &TestContext| match msg {
                CounterMsg::Increment => {
                    *model += 1;
                    Effect::RequestFocus(NodeId::new(9))
                }
            },
            |model: &i32, _shared: &(), _ctx: &TestContext| Scene::text(1_u64, model.to_string()),
        );

        let ctx = TestContext { title: "knopper" };
        let mut model = machine.init(&ctx);
        let effect = machine.update(&mut model, CounterMsg::Increment, &ctx);

        assert_eq!(model, 1);
        assert_eq!(effect, Effect::RequestFocus(NodeId::new(9)));
    }
}
