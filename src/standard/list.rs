use crate::{
    Effect, Machine, NodeId, Role, Scene, SceneBehavior, ScrollOffset, SizeConstraint, Style,
};
use cliffy_core::{Behavior, FromGeometric, GA3, IntoGeometric, behavior};
use core::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ListState {
    pub selected: usize,
    pub scroll: u16,
}

impl IntoGeometric for ListState {
    fn into_geometric(self) -> GA3 {
        GA3::zero()
    }
}

impl FromGeometric for ListState {
    fn from_geometric(_mv: &GA3) -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListMsg<RowMsg> {
    MoveUp,
    MoveDown,
    Select(usize),
    Row(RowMsg),
}

#[derive(Debug, Clone)]
pub struct ListContext<Item> {
    pub items: Vec<Item>,
    pub viewport_height: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListIds {
    pub root: NodeId,
    pub viewport: NodeId,
    pub scroll: NodeId,
    pub column: NodeId,
    pub empty: NodeId,
    pub item_base: NodeId,
    pub marker_base: NodeId,
}

impl Default for ListIds {
    fn default() -> Self {
        Self {
            root: NodeId::new(10_000),
            viewport: NodeId::new(10_001),
            scroll: NodeId::new(10_002),
            column: NodeId::new(10_003),
            empty: NodeId::new(10_004),
            item_base: NodeId::new(11_000),
            marker_base: NodeId::new(12_000),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ListMachine<Item, RowMsg, RenderRow> {
    ids: ListIds,
    render_row: RenderRow,
    _phantom: PhantomData<(Item, RowMsg)>,
}

impl<Item, RowMsg, RenderRow> ListMachine<Item, RowMsg, RenderRow> {
    #[must_use]
    pub fn new(render_row: RenderRow) -> Self {
        Self {
            ids: ListIds::default(),
            render_row,
            _phantom: PhantomData,
        }
    }

    #[must_use]
    pub fn with_ids(mut self, ids: ListIds) -> Self {
        self.ids = ids;
        self
    }

    #[must_use]
    pub fn item_id(&self, index: usize) -> NodeId {
        offset_id(self.ids.item_base, index)
    }

    #[must_use]
    pub fn marker_id(&self, index: usize) -> NodeId {
        offset_id(self.ids.marker_base, index)
    }
}

impl<Item, RowMsg, RenderRow> Machine for ListMachine<Item, RowMsg, RenderRow>
where
    Item: Clone + 'static,
    RowMsg: Clone + 'static,
    RenderRow: Fn(&Item, bool) -> Scene<RowMsg> + Clone + 'static,
{
    type Context = ListContext<Item>;
    type Msg = ListMsg<RowMsg>;
    type Model = ListState;
    type Shared = ();

    fn init(&self, _ctx: &Self::Context) -> Self::Model {
        ListState::default()
    }

    fn update(
        &self,
        model: &mut Self::Model,
        msg: Self::Msg,
        ctx: &Self::Context,
    ) -> Effect<Self::Msg> {
        let len = ctx.items.len();
        if len == 0 {
            model.selected = 0;
            model.scroll = 0;
            return Effect::None;
        }

        match msg {
            ListMsg::MoveUp => {
                model.selected = model.selected.saturating_sub(1);
            }
            ListMsg::MoveDown => {
                model.selected = (model.selected + 1).min(len.saturating_sub(1));
            }
            ListMsg::Select(index) => {
                model.selected = index.min(len.saturating_sub(1));
            }
            ListMsg::Row(_) => return Effect::None,
        }

        model.scroll = scroll_for(model.selected, model.scroll, ctx.viewport_height);
        Effect::RequestFocus(self.marker_id(model.selected))
    }

    fn project(
        &self,
        model: Behavior<Self::Model>,
        _shared: Behavior<Self::Shared>,
        ctx: &Self::Context,
    ) -> SceneBehavior<Self::Msg> {
        let scene = behavior(build_scene(
            &self.ids,
            &self.render_row,
            &ctx.items,
            &model.sample(),
            ctx.viewport_height,
        ));

        let ids = self.ids;
        let render_row = self.render_row.clone();
        let items = ctx.items.clone();
        let viewport_height = ctx.viewport_height;
        let scene_for_model = scene.clone();
        model.subscribe(move |next_model| {
            scene_for_model.set(build_scene(
                &ids,
                &render_row,
                &items,
                next_model,
                viewport_height,
            ));
        });

        scene
    }
}

fn build_scene<Item, RowMsg, RenderRow>(
    ids: &ListIds,
    render_row: &RenderRow,
    items: &[Item],
    state: &ListState,
    viewport_height: u16,
) -> Scene<ListMsg<RowMsg>>
where
    Item: Clone,
    RowMsg: Clone,
    RenderRow: Fn(&Item, bool) -> Scene<RowMsg>,
{
    let selected = clamp_selected(state.selected, items.len());
    let scroll = scroll_for(selected, state.scroll, viewport_height);

    let content = if items.is_empty() {
        Scene::text(ids.empty, "(empty list)")
    } else {
        Scene::column(
            ids.column,
            items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    let is_selected = index == selected;
                    let marker = Scene::text(
                        offset_id(ids.marker_base, index),
                        if is_selected { ">" } else { " " },
                    )
                    .with_role(Role::ListItem)
                    .focusable()
                    .on_activate(ListMsg::Select(index));

                    let row = render_row(item, is_selected).map_msg(&ListMsg::Row);
                    let mut scene = Scene::row(offset_id(ids.item_base, index), vec![marker, row])
                        .with_role(Role::ListItem);

                    if is_selected {
                        scene = scene.with_style(Style::PLAIN.bold().underlined());
                    }

                    scene
                })
                .collect::<Vec<_>>(),
        )
    };

    Scene::sized(
        ids.root,
        SizeConstraint::height(viewport_height.max(1)),
        Scene::viewport(
            ids.viewport,
            Scene::scroll(ids.scroll, ScrollOffset::new(0, scroll), content),
        ),
    )
}

fn offset_id(base: NodeId, index: usize) -> NodeId {
    NodeId::new(
        base.get()
            .saturating_add(u64::try_from(index).unwrap_or(u64::MAX)),
    )
}

fn clamp_selected(selected: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        selected.min(len.saturating_sub(1))
    }
}

fn scroll_for(selected: usize, current_scroll: u16, viewport_height: u16) -> u16 {
    let viewport_height = viewport_height.max(1);
    let current_scroll = usize::from(current_scroll);
    let viewport_height = usize::from(viewport_height);

    if selected < current_scroll {
        u16::try_from(selected).unwrap_or(u16::MAX)
    } else if selected >= current_scroll.saturating_add(viewport_height) {
        u16::try_from(selected + 1 - viewport_height).unwrap_or(u16::MAX)
    } else {
        u16::try_from(current_scroll).unwrap_or(u16::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Runtime, activation_message, layout::Rect, render::RenderOp};

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum RowMsg {
        Open(&'static str),
    }

    #[test]
    fn list_machine_renders_rich_rows_and_scrolls_to_keep_selection_visible() {
        let machine = ListMachine::new(|item: &&str, selected| {
            Scene::<RowMsg>::row(
                item.len() as u64 + 200,
                vec![
                    Scene::text(item.len() as u64 + 300, *item),
                    Scene::text(
                        item.len() as u64 + 400,
                        if selected { " [selected]" } else { "" },
                    ),
                ],
            )
        });
        let ctx = ListContext {
            items: vec!["alpha", "beta", "gamma", "delta"],
            viewport_height: 2,
        };
        let mut runtime = Runtime::new(machine.clone(), ctx, ());

        assert_eq!(
            runtime.render_ops(Rect::new(0, 0, 24, 2)),
            vec![
                RenderOp::DrawText {
                    id: machine.marker_id(0),
                    rect: Rect::new(0, 0, 1, 1),
                    content: ">".into(),
                    style: Style::PLAIN.bold().underlined(),
                },
                RenderOp::DrawText {
                    id: NodeId::new(305),
                    rect: Rect::new(1, 0, 5, 1),
                    content: "alpha".into(),
                    style: Style::PLAIN.bold().underlined(),
                },
                RenderOp::DrawText {
                    id: NodeId::new(405),
                    rect: Rect::new(6, 0, 11, 1),
                    content: " [selected]".into(),
                    style: Style::PLAIN.bold().underlined(),
                },
                RenderOp::DrawText {
                    id: machine.marker_id(1),
                    rect: Rect::new(0, 1, 1, 1),
                    content: " ".into(),
                    style: Style::PLAIN,
                },
                RenderOp::DrawText {
                    id: NodeId::new(304),
                    rect: Rect::new(1, 1, 4, 1),
                    content: "beta".into(),
                    style: Style::PLAIN,
                },
            ]
        );

        runtime.send(ListMsg::MoveDown);
        runtime.send(ListMsg::MoveDown);

        assert_eq!(
            runtime.model(),
            ListState {
                selected: 2,
                scroll: 1
            }
        );
        assert_eq!(
            runtime
                .focus()
                .current()
                .and_then(crate::FocusPath::current),
            Some(machine.marker_id(2))
        );
        assert_eq!(
            runtime.render_ops(Rect::new(0, 0, 24, 2)),
            vec![
                RenderOp::DrawText {
                    id: machine.marker_id(1),
                    rect: Rect::new(0, 0, 1, 1),
                    content: " ".into(),
                    style: Style::PLAIN,
                },
                RenderOp::DrawText {
                    id: NodeId::new(304),
                    rect: Rect::new(1, 0, 4, 1),
                    content: "beta".into(),
                    style: Style::PLAIN,
                },
                RenderOp::DrawText {
                    id: machine.marker_id(2),
                    rect: Rect::new(0, 1, 1, 1),
                    content: ">".into(),
                    style: Style::PLAIN.bold().underlined(),
                },
                RenderOp::DrawText {
                    id: NodeId::new(305),
                    rect: Rect::new(1, 1, 5, 1),
                    content: "gamma".into(),
                    style: Style::PLAIN.bold().underlined(),
                },
                RenderOp::DrawText {
                    id: NodeId::new(405),
                    rect: Rect::new(6, 1, 11, 1),
                    content: " [selected]".into(),
                    style: Style::PLAIN.bold().underlined(),
                },
            ]
        );
    }

    #[test]
    fn selecting_item_via_activation_updates_selection() {
        let machine = ListMachine::new(|item: &&str, _selected| {
            Scene::<RowMsg>::text(item.len() as u64, *item)
        });
        let ctx = ListContext {
            items: vec!["alpha", "beta", "gamma"],
            viewport_height: 3,
        };
        let mut runtime = Runtime::new(machine.clone(), ctx, ());

        runtime.dispatch(crate::RuntimeEvent::Activate(machine.marker_id(2)));

        assert_eq!(
            runtime.model(),
            ListState {
                selected: 2,
                scroll: 0
            }
        );
        assert_eq!(
            runtime
                .focus()
                .current()
                .and_then(crate::FocusPath::current),
            Some(machine.marker_id(2))
        );
    }

    #[test]
    fn row_messages_are_lifted_through_list_message_type() {
        let machine = ListMachine::new(|item: &&'static str, _selected| {
            Scene::text(item.len() as u64 + 500, *item).on_activate(RowMsg::Open(item))
        });
        let ctx = ListContext {
            items: vec!["alpha", "beta"],
            viewport_height: 2,
        };
        let runtime = Runtime::new(machine, ctx, ());
        let scene = runtime.scene().sample();

        assert_eq!(
            activation_message(&scene, NodeId::new(505)),
            Some(ListMsg::Row(RowMsg::Open("alpha")))
        );
    }
}
