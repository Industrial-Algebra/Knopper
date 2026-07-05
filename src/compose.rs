use crate::{Effect, FocusState, Machine, NodeId, Scene};

#[must_use]
pub fn child_has_focus(focus: &FocusState, child_root: NodeId) -> bool {
    focus
        .current()
        .is_some_and(|path| path.as_slice().contains(&child_root))
}

#[must_use]
pub fn dispatch_if_focused<Msg>(
    focus: &FocusState,
    child_root: NodeId,
    msg: Option<Msg>,
) -> Option<Msg> {
    child_has_focus(focus, child_root).then_some(msg).flatten()
}

#[must_use]
pub fn trap_focus(focus: &FocusState, scope_root: NodeId, fallback: NodeId) -> Option<NodeId> {
    (!child_has_focus(focus, scope_root)).then_some(fallback)
}

#[must_use]
pub fn map_effect<Msg, NextMsg>(
    effect: Effect<Msg>,
    f: &impl Fn(Msg) -> NextMsg,
) -> Effect<NextMsg> {
    match effect {
        Effect::None => Effect::None,
        Effect::Emit(msg) => Effect::Emit(f(msg)),
        Effect::Batch(effects) => {
            Effect::Batch(effects.into_iter().map(|e| map_effect(e, f)).collect())
        }
        Effect::RequestFocus(id) => Effect::RequestFocus(id),
    }
}

#[must_use]
pub fn update_child<M, ParentMsg>(
    machine: &M,
    model: &mut M::Model,
    msg: M::Msg,
    ctx: &M::Context,
    lift: &impl Fn(M::Msg) -> ParentMsg,
) -> Effect<ParentMsg>
where
    M: Machine,
{
    map_effect(machine.update(model, msg, ctx), lift)
}

#[must_use]
pub fn project_child<M, ParentMsg>(
    machine: &M,
    model: &M::Model,
    shared: &M::Shared,
    ctx: &M::Context,
    lift: &impl Fn(M::Msg) -> ParentMsg,
) -> Scene<ParentMsg>
where
    M: Machine,
    M::Msg: Clone + 'static,
    M::Model: Clone + cliffy_core::IntoGeometric + cliffy_core::FromGeometric + 'static,
    M::Shared: Clone + cliffy_core::IntoGeometric + cliffy_core::FromGeometric + 'static,
{
    machine.project_once(model, shared, ctx).map_msg(lift)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Effect, FocusPath, FocusState, PureMachine, Role, Scene};

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Ctx;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum ChildMsg {
        Inc,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum ParentMsg {
        Child(ChildMsg),
    }

    #[test]
    fn detects_when_focus_is_within_child_subtree() {
        let mut focus = FocusState::new();
        focus.set(FocusPath::from_vec(vec![
            NodeId::new(1),
            NodeId::new(10),
            NodeId::new(11),
        ]));

        assert!(child_has_focus(&focus, NodeId::new(10)));
        assert!(!child_has_focus(&focus, NodeId::new(20)));
        assert_eq!(
            dispatch_if_focused(&focus, NodeId::new(10), Some("ok")),
            Some("ok")
        );
        assert_eq!(
            dispatch_if_focused::<&str>(&focus, NodeId::new(20), Some("ok")),
            None
        );
        assert_eq!(trap_focus(&focus, NodeId::new(10), NodeId::new(11)), None);
        assert_eq!(
            trap_focus(&focus, NodeId::new(20), NodeId::new(11)),
            Some(NodeId::new(11))
        );
    }

    #[test]
    fn maps_child_effects_into_parent_space() {
        let machine = PureMachine::new(
            |_ctx: &Ctx| 0_i32,
            |model: &mut i32, msg: ChildMsg, _ctx: &Ctx| {
                match msg {
                    ChildMsg::Inc => *model += 1,
                }
                Effect::Emit(ChildMsg::Inc)
            },
            |model: &i32, _shared: &(), _ctx: &Ctx| Scene::text(1_u64, model.to_string()),
        );

        let mut model = 0;
        let effect = update_child(&machine, &mut model, ChildMsg::Inc, &Ctx, &ParentMsg::Child);

        assert_eq!(model, 1);
        assert_eq!(effect, Effect::Emit(ParentMsg::Child(ChildMsg::Inc)));
    }

    #[test]
    fn projects_child_scene_into_parent_space() {
        let machine = PureMachine::new(
            |_ctx: &Ctx| 0_i32,
            |_model: &mut i32, _msg: ChildMsg, _ctx: &Ctx| Effect::None,
            |_model: &i32, _shared: &(), _ctx: &Ctx| {
                Scene::text(7_u64, "child")
                    .with_role(Role::StatusLine)
                    .on_activate(ChildMsg::Inc)
            },
        );

        let scene = project_child(&machine, &0, &(), &Ctx, &ParentMsg::Child);

        assert_eq!(
            scene,
            Scene::text(7_u64, "child")
                .with_role(Role::StatusLine)
                .on_activate(ParentMsg::Child(ChildMsg::Inc))
        );
    }
}
