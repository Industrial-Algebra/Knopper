use crate::{
    Color, Effect, FocusState, KeyEvent, LayoutNode, Machine, NodeId, Scene, SceneBehavior,
    SizeConstraint, Style, child_has_focus, dispatch_if_focused, modal_key_msg, modal_scene,
    project_child,
    standard::{
        input::{InputContext, InputMachine, InputMsg, InputState},
        list::{ListContext, ListMachine, ListMsg, ListState},
        modal::{ModalFocusConfig, ModalIds, ModalMsg},
    },
    update_child,
};
use cliffy_core::{Behavior, FromGeometric, GA3, IntoGeometric, behavior};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteState {
    pub open: bool,
    pub input: InputState,
    pub list: ListState,
    pub filtered: Vec<usize>,
    pub committed: Option<usize>,
}

impl Default for CommandPaletteState {
    fn default() -> Self {
        Self {
            open: true,
            input: InputState::default(),
            list: ListState::default(),
            filtered: Vec::new(),
            committed: None,
        }
    }
}

impl IntoGeometric for CommandPaletteState {
    fn into_geometric(self) -> GA3 {
        GA3::zero()
    }
}

impl FromGeometric for CommandPaletteState {
    fn from_geometric(_mv: &GA3) -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandPaletteMsg {
    Key(KeyEvent),
    Input(InputMsg),
    List(ListMsg<()>),
    FocusInput,
    FocusList,
    Dismiss,
}

#[derive(Debug, Clone)]
pub struct CommandPaletteContext {
    pub items: Vec<String>,
    pub width: u16,
    pub list_height: u16,
    pub input: InputContext,
}

impl Default for CommandPaletteContext {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            width: 32,
            list_height: 6,
            input: InputContext {
                root_id: NodeId::new(30_000),
                field_id: NodeId::new(30_001),
                width: 30,
                placeholder: "Type a command".into(),
            },
        }
    }
}

type PaletteRowRenderer = fn(&String, bool) -> Scene<()>;

#[derive(Debug, Clone)]
pub struct CommandPaletteMachine {
    input: InputMachine,
    list: ListMachine<String, (), PaletteRowRenderer>,
}

impl Default for CommandPaletteMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPaletteMachine {
    pub const MODAL_IDS: ModalIds = ModalIds {
        root: NodeId::new(30_203),
        dialog: NodeId::new(30_207),
        backdrop: NodeId::new(30_208),
        frame: NodeId::new(30_206),
    };

    #[must_use]
    pub fn new() -> Self {
        Self {
            input: InputMachine,
            list: ListMachine::new(render_palette_item as PaletteRowRenderer),
        }
    }

    #[must_use]
    pub fn list_root_id(&self) -> NodeId {
        self.list.root_id()
    }

    #[must_use]
    pub fn input_root_id(&self, ctx: &CommandPaletteContext) -> NodeId {
        self.input.root_id(&ctx.input)
    }

    pub const MAIN_SCOPE: &'static str = "command-palette-main";

    #[must_use]
    pub fn focus_order(
        &self,
        ctx: &CommandPaletteContext,
        state: &CommandPaletteState,
    ) -> crate::FocusOrder {
        let body = self.focus_scope_scene(ctx, state);
        crate::FocusOrder::collect_from_scope(&body, Self::MAIN_SCOPE)
            .unwrap_or_else(|| crate::FocusOrder::collect_from_scene(&body))
    }

    #[must_use]
    fn focus_scope_scene(
        &self,
        ctx: &CommandPaletteContext,
        state: &CommandPaletteState,
    ) -> Scene<CommandPaletteMsg> {
        let input_scene = project_child(
            &self.input,
            &state.input,
            &(),
            &ctx.input,
            &CommandPaletteMsg::Input,
        );
        let list_scene = project_child(
            &self.list,
            &state.list,
            &(),
            &filtered_list_context(ctx, &state.filtered),
            &CommandPaletteMsg::List,
        );
        Scene::focus_scope_with_policy(
            30_299_u64,
            Self::MAIN_SCOPE,
            crate::FocusScopePolicy::Trap,
            Scene::column(30_298_u64, vec![input_scene, list_scene]),
        )
    }

    #[must_use]
    pub fn modal_focus_config(
        &self,
        ctx: &CommandPaletteContext,
        state: &CommandPaletteState,
    ) -> ModalFocusConfig<CommandPaletteMsg> {
        ModalFocusConfig {
            scope_root: Self::MODAL_IDS.root,
            focus_order: self.focus_order(ctx, state),
            primary_focus: self.input_root_id(ctx),
            tab_forward: CommandPaletteMsg::FocusList,
            tab_backward: CommandPaletteMsg::FocusInput,
        }
    }

    fn filtered_items(&self, ctx: &CommandPaletteContext, query: &str) -> Vec<usize> {
        let needle = query.to_lowercase();
        ctx.items
            .iter()
            .enumerate()
            .filter(|(_, item)| needle.is_empty() || item.to_lowercase().contains(&needle))
            .map(|(index, _)| index)
            .collect()
    }

    fn sync_filtered(&self, model: &mut CommandPaletteState, ctx: &CommandPaletteContext) {
        model.filtered = self.filtered_items(ctx, &model.input.value);
        if model.filtered.is_empty() {
            model.list = ListState::default();
            if model
                .committed
                .is_some_and(|index| !model.filtered.contains(&index))
            {
                model.committed = None;
            }
            return;
        }

        model.list.selected = model
            .list
            .selected
            .min(model.filtered.len().saturating_sub(1));
        model.list.scroll =
            adjusted_scroll(model.list.selected, model.list.scroll, ctx.list_height);
        if let Some(committed) = model.committed
            && !model.filtered.contains(&committed)
        {
            model.committed = None;
        }
    }

    #[must_use]
    pub fn key_msg(
        &self,
        focus: &FocusState,
        state: &CommandPaletteState,
        event: KeyEvent,
        ctx: &CommandPaletteContext,
    ) -> Option<CommandPaletteMsg> {
        if !state.open {
            return None;
        }

        if let Some(modal_msg) = modal_key_msg(
            focus,
            state.open,
            event,
            &self.modal_focus_config(ctx, state),
        ) {
            return Some(match modal_msg {
                ModalMsg::Inner(msg) => msg,
                ModalMsg::FocusPrimary => CommandPaletteMsg::FocusInput,
                ModalMsg::Dismiss => CommandPaletteMsg::Dismiss,
            });
        }

        if child_has_focus(focus, self.input_root_id(ctx)) {
            self.input.key_msg(event).map(CommandPaletteMsg::Input)
        } else {
            dispatch_if_focused(
                focus,
                self.list.root_id(),
                self.list
                    .key_msg(&state.list, event)
                    .map(CommandPaletteMsg::List),
            )
        }
    }
}

impl Machine for CommandPaletteMachine {
    type Context = CommandPaletteContext;
    type Msg = CommandPaletteMsg;
    type Model = CommandPaletteState;
    type Shared = ();

    fn init(&self, ctx: &Self::Context) -> Self::Model {
        CommandPaletteState {
            filtered: self.filtered_items(ctx, ""),
            ..CommandPaletteState::default()
        }
    }

    fn update(
        &self,
        model: &mut Self::Model,
        msg: Self::Msg,
        ctx: &Self::Context,
    ) -> Effect<Self::Msg> {
        match msg {
            CommandPaletteMsg::Key(event) => self
                .key_msg(&FocusState::default(), model, event, ctx)
                .map_or(Effect::None, Effect::Emit),
            CommandPaletteMsg::FocusInput => Effect::RequestFocus(self.input_root_id(ctx)),
            CommandPaletteMsg::FocusList => Effect::RequestFocus(self.list.root_id()),
            CommandPaletteMsg::Dismiss => {
                model.open = false;
                Effect::None
            }
            CommandPaletteMsg::Input(input_msg) => {
                let effect = update_child(
                    &self.input,
                    &mut model.input,
                    input_msg,
                    &ctx.input,
                    &CommandPaletteMsg::Input,
                );
                self.sync_filtered(model, ctx);
                effect
            }
            CommandPaletteMsg::List(list_msg) => {
                if let ListMsg::Commit(filtered_index) = list_msg.clone() {
                    model.list.selected =
                        filtered_index.min(model.filtered.len().saturating_sub(1));
                    if let Some(actual_index) = model.filtered.get(model.list.selected).copied() {
                        model.committed = Some(actual_index);
                    }
                }

                let list_ctx = filtered_list_context(ctx, &model.filtered);
                update_child(
                    &self.list,
                    &mut model.list,
                    list_msg,
                    &list_ctx,
                    &CommandPaletteMsg::List,
                )
            }
        }
    }

    fn project_once(
        &self,
        model: &Self::Model,
        _shared: &Self::Shared,
        ctx: &Self::Context,
    ) -> Scene<Self::Msg> {
        if !model.open {
            return Scene::Empty;
        }

        let body = self.focus_scope_scene(ctx, model);
        let detail = Scene::text(
            30_100_u64,
            model
                .committed
                .and_then(|index| ctx.items.get(index))
                .map_or_else(
                    || "committed:<none>".to_string(),
                    |item| format!("committed:{item}"),
                ),
        );
        modal_scene(
            Self::MODAL_IDS,
            Scene::border(
                30_201_u64,
                Scene::sized(
                    30_202_u64,
                    SizeConstraint::width(ctx.width),
                    Scene::column(30_200_u64, vec![body, detail])
                        .with_style(Style::PLAIN.bg(Color::Ansi(0))),
                ),
            )
            .with_style(Style::PLAIN.bg(Color::Ansi(0))),
        )
        .map_msg(&|msg| match msg {
            ModalMsg::Inner(inner) => inner,
            ModalMsg::FocusPrimary => CommandPaletteMsg::FocusInput,
            ModalMsg::Dismiss => CommandPaletteMsg::Dismiss,
        })
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
            scene_for_model.set(CommandPaletteMachine::new().project_once(next_model, &(), &ctx));
        });
        scene
    }

    fn cursor_position(
        &self,
        model: &Self::Model,
        _shared: &Self::Shared,
        ctx: &Self::Context,
        layout: &LayoutNode,
    ) -> Option<(u16, u16)> {
        self.input
            .cursor_position(&model.input, &(), &ctx.input, layout)
    }
}

fn render_palette_item(item: &String, selected: bool) -> Scene<()> {
    let content = if selected {
        format!("> {item}")
    } else {
        format!("  {item}")
    };
    Scene::text(NodeId::new(31_000 + item.len() as u64), content)
}

fn filtered_list_context(ctx: &CommandPaletteContext, filtered: &[usize]) -> ListContext<String> {
    ListContext {
        items: filtered
            .iter()
            .filter_map(|index| ctx.items.get(*index).cloned())
            .collect(),
        viewport_height: ctx.list_height,
    }
}

fn adjusted_scroll(selected: usize, current_scroll: u16, viewport_height: u16) -> u16 {
    let viewport_height = usize::from(viewport_height.max(1));
    let current_scroll = usize::from(current_scroll);
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
    use crate::{FocusPath, Key, NodeId, RenderOp, Runtime, activation_message, layout::Rect};

    fn ctx() -> CommandPaletteContext {
        CommandPaletteContext {
            items: vec![
                "Open File".into(),
                "Close Window".into(),
                "Toggle Sidebar".into(),
            ],
            width: 32,
            list_height: 3,
            ..CommandPaletteContext::default()
        }
    }

    #[test]
    fn typing_filters_palette_items() {
        let mut runtime = Runtime::new(CommandPaletteMachine::new(), ctx(), ());
        runtime.send(CommandPaletteMsg::Input(InputMsg::Insert('o')));
        runtime.send(CommandPaletteMsg::Input(InputMsg::Insert('p')));

        assert_eq!(runtime.model().filtered, vec![0]);
        let ops = runtime.render_ops(Rect::new(0, 0, 40, 10));
        assert!(ops.iter().any(|op| matches!(
            op,
            RenderOp::DrawText { content, .. } if content.contains("Open File")
        )));
        assert!(!ops.iter().any(|op| matches!(
            op,
            RenderOp::DrawText { content, .. } if content.contains("Close Window")
        )));
    }

    #[test]
    fn list_commit_tracks_underlying_item_index() {
        let mut runtime = Runtime::new(CommandPaletteMachine::new(), ctx(), ());
        runtime.send(CommandPaletteMsg::Input(InputMsg::Insert('t')));
        runtime.send(CommandPaletteMsg::Input(InputMsg::Insert('o')));
        runtime.send(CommandPaletteMsg::List(ListMsg::Commit(0)));

        assert_eq!(runtime.model().filtered, vec![2]);
        assert_eq!(runtime.model().committed, Some(2));
    }

    #[test]
    fn key_dispatch_respects_focused_child() {
        let machine = CommandPaletteMachine::new();
        let state = machine.init(&ctx());
        let key = KeyEvent {
            key: Key::Char('x'),
            ctrl: false,
            alt: false,
            shift: false,
        };

        let mut input_focus = FocusState::new();
        input_focus.set(FocusPath::from_vec(vec![
            CommandPaletteMachine::MODAL_IDS.root,
            NodeId::new(30_000),
            NodeId::new(30_001),
        ]));
        assert_eq!(
            machine.key_msg(&input_focus, &state, key, &ctx()),
            Some(CommandPaletteMsg::Input(InputMsg::Insert('x')))
        );

        let mut list_focus = FocusState::new();
        list_focus.set(FocusPath::from_vec(vec![
            CommandPaletteMachine::MODAL_IDS.root,
            NodeId::new(10_000),
            NodeId::new(12_000),
        ]));
        assert_eq!(
            machine.key_msg(
                &list_focus,
                &state,
                KeyEvent {
                    key: Key::Down,
                    ctrl: false,
                    alt: false,
                    shift: false,
                },
                &ctx()
            ),
            Some(CommandPaletteMsg::List(ListMsg::MoveDown))
        );
    }

    #[test]
    fn scene_derived_focus_order_matches_input_then_list() {
        let machine = CommandPaletteMachine::new();
        let state = machine.init(&ctx());
        let order = machine.focus_order(&ctx(), &state);

        assert_eq!(
            order.as_slice(),
            &[
                NodeId::new(30_001),
                NodeId::new(12_000),
                NodeId::new(12_001),
                NodeId::new(12_002),
            ]
        );
    }

    #[test]
    fn escape_dismisses_modal() {
        let machine = CommandPaletteMachine::new();
        let mut state = machine.init(&ctx());
        let mut focus = FocusState::new();
        focus.set(FocusPath::from_vec(vec![
            CommandPaletteMachine::MODAL_IDS.root,
            NodeId::new(30_001),
        ]));

        assert_eq!(
            machine.key_msg(
                &focus,
                &state,
                KeyEvent {
                    key: Key::Escape,
                    ctrl: false,
                    alt: false,
                    shift: false,
                },
                &ctx(),
            ),
            Some(CommandPaletteMsg::Dismiss)
        );

        let effect = machine.update(&mut state, CommandPaletteMsg::Dismiss, &ctx());
        assert_eq!(effect, Effect::None);
        assert!(!state.open);
    }

    #[test]
    fn tab_cycles_focus_inside_modal_scope() {
        let machine = CommandPaletteMachine::new();
        let state = machine.init(&ctx());

        let mut input_focus = FocusState::new();
        input_focus.set(FocusPath::from_vec(vec![
            CommandPaletteMachine::MODAL_IDS.root,
            NodeId::new(30_000),
            NodeId::new(30_001),
        ]));
        assert_eq!(
            machine.key_msg(
                &input_focus,
                &state,
                KeyEvent {
                    key: Key::Tab,
                    ctrl: false,
                    alt: false,
                    shift: false,
                },
                &ctx(),
            ),
            Some(CommandPaletteMsg::FocusList)
        );

        let mut list_focus = FocusState::new();
        list_focus.set(FocusPath::from_vec(vec![
            CommandPaletteMachine::MODAL_IDS.root,
            NodeId::new(10_000),
        ]));
        assert_eq!(
            machine.key_msg(
                &list_focus,
                &state,
                KeyEvent {
                    key: Key::Tab,
                    ctrl: false,
                    alt: false,
                    shift: true,
                },
                &ctx(),
            ),
            Some(CommandPaletteMsg::FocusInput)
        );
    }

    #[test]
    fn focus_is_trapped_back_into_modal() {
        let machine = CommandPaletteMachine::new();
        let state = machine.init(&ctx());
        let focus = FocusState::new();

        assert_eq!(
            machine.key_msg(
                &focus,
                &state,
                KeyEvent {
                    key: Key::Down,
                    ctrl: false,
                    alt: false,
                    shift: false,
                },
                &ctx(),
            ),
            Some(CommandPaletteMsg::FocusInput)
        );
    }

    #[test]
    fn backdrop_activation_lifts_dismiss_message() {
        let runtime = Runtime::new(CommandPaletteMachine::new(), ctx(), ());
        let scene = runtime.scene().sample();
        assert_eq!(
            activation_message(&scene, CommandPaletteMachine::MODAL_IDS.backdrop),
            Some(CommandPaletteMsg::Dismiss)
        );
    }

    #[test]
    fn projected_scene_uses_modal_overlay_and_reports_cursor() {
        let mut runtime = Runtime::new(CommandPaletteMachine::new(), ctx(), ());
        runtime.send(CommandPaletteMsg::Input(InputMsg::Insert('x')));
        let scene = runtime.scene().sample();
        let ops = runtime.render_ops(Rect::new(0, 0, 40, 12));

        assert!(matches!(scene, Scene::Stack { .. }));
        assert!(ops.iter().any(|op| matches!(
            op,
            RenderOp::Annotate { label, .. } if label == "modal-backdrop"
        )));
        assert_eq!(runtime.cursor(Rect::new(0, 0, 40, 12)), Some((5, 3)));
        assert_eq!(activation_message(&scene, NodeId::new(20_001)), None);
    }
}
