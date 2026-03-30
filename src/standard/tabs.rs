use crate::{Effect, Key, KeyEvent, Machine, NodeId, Role, Scene, SceneBehavior, Style};
use cliffy_core::{Behavior, FromGeometric, GA3, IntoGeometric, behavior};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TabsState {
    pub selected: usize,
    pub committed: Option<usize>,
}

impl IntoGeometric for TabsState {
    fn into_geometric(self) -> GA3 {
        GA3::zero()
    }
}

impl FromGeometric for TabsState {
    fn from_geometric(_mv: &GA3) -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabsMsg {
    MoveLeft,
    MoveRight,
    Select(usize),
    Commit(usize),
}

#[must_use]
pub fn tabs_key_msg(state: &TabsState, event: KeyEvent) -> Option<TabsMsg> {
    match event.key {
        Key::Left => Some(TabsMsg::MoveLeft),
        Key::Right => Some(TabsMsg::MoveRight),
        Key::Enter => Some(TabsMsg::Commit(state.selected)),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct TabsContext {
    pub root_id: NodeId,
    pub tab_base_id: NodeId,
    pub labels: Vec<String>,
}

impl Default for TabsContext {
    fn default() -> Self {
        Self {
            root_id: NodeId::new(70_000),
            tab_base_id: NodeId::new(70_100),
            labels: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TabsMachine;

impl TabsMachine {
    #[must_use]
    pub fn key_msg(&self, state: &TabsState, event: KeyEvent) -> Option<TabsMsg> {
        tabs_key_msg(state, event)
    }

    #[must_use]
    pub fn root_id(&self, ctx: &TabsContext) -> NodeId {
        ctx.root_id
    }

    #[must_use]
    pub fn tab_id(&self, ctx: &TabsContext, index: usize) -> NodeId {
        NodeId::new(
            ctx.tab_base_id
                .get()
                .saturating_add(u64::try_from(index).unwrap_or(u64::MAX)),
        )
    }
}

impl Machine for TabsMachine {
    type Context = TabsContext;
    type Msg = TabsMsg;
    type Model = TabsState;
    type Shared = ();

    fn init(&self, _ctx: &Self::Context) -> Self::Model {
        TabsState::default()
    }

    fn update(
        &self,
        model: &mut Self::Model,
        msg: Self::Msg,
        ctx: &Self::Context,
    ) -> Effect<Self::Msg> {
        if ctx.labels.is_empty() {
            model.selected = 0;
            model.committed = None;
            return Effect::None;
        }

        let last = ctx.labels.len().saturating_sub(1);
        match msg {
            TabsMsg::MoveLeft => model.selected = model.selected.saturating_sub(1),
            TabsMsg::MoveRight => model.selected = (model.selected + 1).min(last),
            TabsMsg::Select(index) | TabsMsg::Commit(index) => {
                model.selected = index.min(last);
                if matches!(msg, TabsMsg::Commit(_)) {
                    model.committed = Some(model.selected);
                }
            }
        }

        Effect::RequestFocus(self.tab_id(ctx, model.selected))
    }

    fn project_once(
        &self,
        model: &Self::Model,
        _shared: &Self::Shared,
        ctx: &Self::Context,
    ) -> Scene<Self::Msg> {
        if ctx.labels.is_empty() {
            return Scene::text(ctx.root_id, "(no tabs)");
        }

        Scene::row(
            ctx.root_id,
            ctx.labels
                .iter()
                .enumerate()
                .map(|(index, label)| {
                    let selected = index == model.selected;
                    let committed = model.committed == Some(index);
                    let content = if committed {
                        format!("[{label}*]")
                    } else {
                        format!("[{label}]")
                    };
                    let style = if selected {
                        Style::PLAIN.bold().underlined()
                    } else {
                        Style::PLAIN
                    };
                    Scene::text(self.tab_id(ctx, index), content)
                        .with_role(Role::Header)
                        .with_style(style)
                        .focusable()
                        .on_activate(TabsMsg::Commit(index))
                })
                .collect::<Vec<_>>(),
        )
    }

    fn project(
        &self,
        model: Behavior<Self::Model>,
        _shared: Behavior<Self::Shared>,
        ctx: &Self::Context,
    ) -> SceneBehavior<Self::Msg> {
        let scene = behavior(self.project_once(&model.sample(), &(), ctx));
        let scene_for_model = scene.clone();
        let ctx = ctx.clone();
        model.subscribe(move |next_model| {
            scene_for_model.set(TabsMachine.project_once(next_model, &(), &ctx));
        });
        scene
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FocusPath, RenderOp, Runtime, layout::Rect};

    fn ctx() -> TabsContext {
        TabsContext {
            labels: vec!["Home".into(), "Logs".into(), "Settings".into()],
            ..TabsContext::default()
        }
    }

    #[test]
    fn tabs_key_binding_maps_navigation_and_commit() {
        let state = TabsState::default();
        assert_eq!(
            tabs_key_msg(
                &state,
                KeyEvent {
                    key: Key::Left,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }
            ),
            Some(TabsMsg::MoveLeft)
        );
        assert_eq!(
            tabs_key_msg(
                &state,
                KeyEvent {
                    key: Key::Right,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }
            ),
            Some(TabsMsg::MoveRight)
        );
        assert_eq!(
            tabs_key_msg(
                &state,
                KeyEvent {
                    key: Key::Enter,
                    ctrl: false,
                    alt: false,
                    shift: false,
                }
            ),
            Some(TabsMsg::Commit(0))
        );
    }

    #[test]
    fn tabs_machine_moves_selection_and_tracks_commit() {
        let machine = TabsMachine;
        let mut runtime = Runtime::new(machine, ctx(), ());

        runtime.send(TabsMsg::MoveRight);
        runtime.send(TabsMsg::Commit(1));

        assert_eq!(
            runtime.model(),
            TabsState {
                selected: 1,
                committed: Some(1)
            }
        );
    }

    #[test]
    fn tabs_machine_requests_focus_for_selected_tab() {
        let machine = TabsMachine;
        let mut runtime = Runtime::new(machine.clone(), ctx(), ());

        runtime.send(TabsMsg::Select(2));

        assert_eq!(
            runtime.focus().current(),
            Some(&FocusPath::from_vec(vec![
                NodeId::new(70_000),
                NodeId::new(70_102)
            ]))
        );
    }

    #[test]
    fn tabs_machine_renders_all_labels() {
        let machine = TabsMachine;
        let runtime = Runtime::new(machine, ctx(), ());
        let ops = runtime.render_ops(Rect::new(0, 0, 40, 1));

        assert!(
            ops.iter()
                .any(|op| matches!(op, RenderOp::DrawText { content, .. } if content == "[Home]"))
        );
        assert!(
            ops.iter()
                .any(|op| matches!(op, RenderOp::DrawText { content, .. } if content == "[Logs]"))
        );
        assert!(
            ops.iter().any(
                |op| matches!(op, RenderOp::DrawText { content, .. } if content == "[Settings]")
            )
        );
    }
}
