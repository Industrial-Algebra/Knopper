use crate::{
    Effect, FocusState, KeyEvent, Machine, Scene, SceneBehavior, Style, dispatch_if_focused,
    project_child,
    standard::list::{ListContext, ListMachine, ListMsg, ListState},
    update_child,
};
use cliffy_core::{Behavior, FromGeometric, GA3, IntoGeometric, behavior};
use core::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ListDetailState {
    pub list: ListState,
    pub committed: Option<usize>,
}

impl IntoGeometric for ListDetailState {
    fn into_geometric(self) -> GA3 {
        GA3::zero()
    }
}

impl FromGeometric for ListDetailState {
    fn from_geometric(_mv: &GA3) -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListDetailMsg<RowMsg> {
    Key(KeyEvent),
    List(ListMsg<RowMsg>),
}

#[derive(Debug, Clone)]
pub struct ListDetailContext<Item> {
    pub items: Vec<Item>,
    pub list_width: u16,
    pub list_viewport_height: u16,
}

#[derive(Debug, Clone)]
pub struct ListDetailMachine<Item, RowMsg, RenderRow, RenderDetail> {
    list: ListMachine<Item, RowMsg, RenderRow>,
    render_detail: RenderDetail,
    _phantom: PhantomData<(Item, RowMsg)>,
}

impl<Item, RowMsg, RenderRow, RenderDetail>
    ListDetailMachine<Item, RowMsg, RenderRow, RenderDetail>
{
    #[must_use]
    pub fn new(list: ListMachine<Item, RowMsg, RenderRow>, render_detail: RenderDetail) -> Self {
        Self {
            list,
            render_detail,
            _phantom: PhantomData,
        }
    }

    #[must_use]
    pub fn key_msg(
        &self,
        focus: &FocusState,
        state: &ListDetailState,
        event: KeyEvent,
    ) -> Option<ListDetailMsg<RowMsg>> {
        dispatch_if_focused(
            focus,
            self.list.root_id(),
            self.list
                .key_msg(&state.list, event)
                .map(ListDetailMsg::List),
        )
    }
}

impl<Item, RowMsg, RenderRow, RenderDetail> Machine
    for ListDetailMachine<Item, RowMsg, RenderRow, RenderDetail>
where
    Item: Clone + 'static,
    RowMsg: Clone + 'static,
    RenderRow: Fn(&Item, bool) -> Scene<RowMsg> + Clone + 'static,
    RenderDetail:
        Fn(Option<&Item>, Option<&Item>) -> Scene<ListDetailMsg<RowMsg>> + Clone + 'static,
{
    type Context = ListDetailContext<Item>;
    type Msg = ListDetailMsg<RowMsg>;
    type Model = ListDetailState;
    type Shared = ();

    fn init(&self, _ctx: &Self::Context) -> Self::Model {
        ListDetailState::default()
    }

    fn update(
        &self,
        model: &mut Self::Model,
        msg: Self::Msg,
        ctx: &Self::Context,
    ) -> Effect<Self::Msg> {
        match msg {
            ListDetailMsg::Key(event) => self
                .list
                .key_msg(&model.list, event)
                .map_or(Effect::None, |msg| Effect::Emit(ListDetailMsg::List(msg))),
            ListDetailMsg::List(list_msg) => {
                if matches!(list_msg, ListMsg::Commit(index) if ctx.items.get(index).is_some())
                    && let ListMsg::Commit(index) = list_msg.clone()
                {
                    model.committed = Some(index);
                }

                let list_ctx = list_context(ctx);
                update_child(
                    &self.list,
                    &mut model.list,
                    list_msg,
                    &list_ctx,
                    &ListDetailMsg::List,
                )
            }
        }
    }

    fn project(
        &self,
        model: Behavior<Self::Model>,
        _shared: Behavior<Self::Shared>,
        ctx: &Self::Context,
    ) -> SceneBehavior<Self::Msg> {
        let scene = behavior(build_scene(
            &self.list,
            &self.render_detail,
            &model.sample(),
            ctx,
        ));

        let list = self.list.clone();
        let render_detail = self.render_detail.clone();
        let ctx = ctx.clone();
        let scene_for_model = scene.clone();
        model.subscribe(move |next_model| {
            scene_for_model.set(build_scene(&list, &render_detail, next_model, &ctx));
        });

        scene
    }
}

fn list_context<Item: Clone>(ctx: &ListDetailContext<Item>) -> ListContext<Item> {
    ListContext {
        items: ctx.items.clone(),
        viewport_height: ctx.list_viewport_height,
    }
}

fn build_scene<Item, RowMsg, RenderRow, RenderDetail>(
    list: &ListMachine<Item, RowMsg, RenderRow>,
    render_detail: &RenderDetail,
    state: &ListDetailState,
    ctx: &ListDetailContext<Item>,
) -> Scene<ListDetailMsg<RowMsg>>
where
    Item: Clone + 'static,
    RowMsg: Clone + 'static,
    RenderRow: Fn(&Item, bool) -> Scene<RowMsg> + Clone + 'static,
    RenderDetail: Fn(Option<&Item>, Option<&Item>) -> Scene<ListDetailMsg<RowMsg>>,
{
    let selected = ctx.items.get(state.list.selected);
    let committed = state.committed.and_then(|index| ctx.items.get(index));
    let list_scene = project_child(
        list,
        &state.list,
        &(),
        &list_context(ctx),
        &ListDetailMsg::List,
    );
    let detail_scene = render_detail(selected, committed);

    Scene::row(
        90_000_u64,
        vec![
            Scene::border(
                90_001_u64,
                Scene::sized(
                    90_002_u64,
                    crate::SizeConstraint::width(ctx.list_width),
                    list_scene,
                ),
            ),
            Scene::border(90_003_u64, detail_scene).with_style(Style::PLAIN),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FocusPath, FocusState, ListMachine, NodeId, Runtime, layout::Rect};

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum RowMsg {
        Open(&'static str),
    }

    fn render_row(item: &&'static str, selected: bool) -> Scene<RowMsg> {
        Scene::<RowMsg>::text(
            item.len() as u64 + 1_000,
            if selected {
                format!("{item} *")
            } else {
                (*item).to_string()
            },
        )
        .on_activate(RowMsg::Open(item))
    }

    fn render_detail(
        selected: Option<&&'static str>,
        committed: Option<&&'static str>,
    ) -> Scene<ListDetailMsg<RowMsg>> {
        Scene::column(
            91_000_u64,
            vec![
                Scene::text(
                    91_001_u64,
                    format!("selected:{}", selected.copied().unwrap_or("<none>")),
                ),
                Scene::text(
                    91_002_u64,
                    format!("committed:{}", committed.copied().unwrap_or("<none>")),
                ),
            ],
        )
    }

    type RowRenderer = fn(&&'static str, bool) -> Scene<RowMsg>;
    type DetailRenderer =
        fn(Option<&&'static str>, Option<&&'static str>) -> Scene<ListDetailMsg<RowMsg>>;
    type DemoMachine = ListDetailMachine<&'static str, RowMsg, RowRenderer, DetailRenderer>;

    fn machine() -> DemoMachine {
        let list = ListMachine::new(render_row as RowRenderer);
        ListDetailMachine::new(list, render_detail as DetailRenderer)
    }

    #[test]
    fn key_dispatch_is_scoped_by_focus() {
        let machine = machine();
        let state = ListDetailState::default();
        let key = KeyEvent {
            key: crate::Key::Down,
            ctrl: false,
            alt: false,
            shift: false,
        };

        let mut focus = FocusState::new();
        focus.set(FocusPath::from_vec(vec![
            NodeId::new(90_000),
            NodeId::new(10_000),
        ]));
        assert_eq!(
            machine.key_msg(&focus, &state, key),
            Some(ListDetailMsg::List(ListMsg::MoveDown))
        );

        let mut other_focus = FocusState::new();
        other_focus.set(FocusPath::from_vec(vec![
            NodeId::new(90_000),
            NodeId::new(90_003),
        ]));
        assert_eq!(machine.key_msg(&other_focus, &state, key), None);
    }

    #[test]
    fn key_messages_drive_selection_and_commit_detail() {
        let mut runtime = Runtime::new(
            machine(),
            ListDetailContext {
                items: vec!["alpha", "beta", "gamma"],
                list_width: 12,
                list_viewport_height: 2,
            },
            (),
        );

        runtime.send(ListDetailMsg::Key(KeyEvent {
            key: crate::Key::Down,
            ctrl: false,
            alt: false,
            shift: false,
        }));
        assert_eq!(runtime.model().list.selected, 1);
        assert_eq!(runtime.model().committed, None);

        runtime.send(ListDetailMsg::Key(KeyEvent {
            key: crate::Key::Enter,
            ctrl: false,
            alt: false,
            shift: false,
        }));
        assert_eq!(runtime.model().committed, Some(1));
    }

    #[test]
    fn scene_shows_selected_and_committed_detail() {
        let mut runtime = Runtime::new(
            machine(),
            ListDetailContext {
                items: vec!["alpha", "beta", "gamma"],
                list_width: 12,
                list_viewport_height: 2,
            },
            (),
        );

        runtime.send(ListDetailMsg::Key(KeyEvent {
            key: crate::Key::Down,
            ctrl: false,
            alt: false,
            shift: false,
        }));
        runtime.send(ListDetailMsg::Key(KeyEvent {
            key: crate::Key::Enter,
            ctrl: false,
            alt: false,
            shift: false,
        }));

        let ops = runtime.render_ops(Rect::new(0, 0, 40, 6));
        assert!(ops.iter().any(|op| matches!(
            op,
            crate::RenderOp::DrawText { content, .. } if content == "selected:beta"
        )));
        assert!(ops.iter().any(|op| matches!(
            op,
            crate::RenderOp::DrawText { content, .. } if content == "committed:beta"
        )));
    }

    #[test]
    fn row_messages_lift_through_demo_machine() {
        let runtime = Runtime::new(
            machine(),
            ListDetailContext {
                items: vec!["alpha", "beta"],
                list_width: 12,
                list_viewport_height: 2,
            },
            (),
        );
        let scene = runtime.scene().sample();

        assert_eq!(
            crate::activation_message(&scene, crate::NodeId::new(1_005)),
            Some(ListDetailMsg::List(ListMsg::Row(RowMsg::Open("alpha"))))
        );
    }
}
