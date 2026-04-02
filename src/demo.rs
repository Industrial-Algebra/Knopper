use crate::{
    Color, Effect, FocusPath, FocusScopePolicy, FocusState, Key, KeyEvent, Machine, Scene,
    SceneBehavior, SizeConstraint, Style, child_has_focus, dispatch_if_focused, project_child,
    standard::{
        button::{ButtonContext, ButtonMachine, ButtonMsg, ButtonState},
        command_palette::{
            CommandPaletteContext, CommandPaletteMachine, CommandPaletteMsg, CommandPaletteState,
        },
        list::{ListContext, ListMachine, ListMsg, ListState},
        tabs::{TabsContext, TabsMachine, TabsMsg, TabsState},
        textarea::{TextareaContext, TextareaMachine, TextareaMsg, TextareaState},
        toggle::{ToggleContext, ToggleMachine, ToggleMsg, ToggleState},
    },
    update_child,
};
use cliffy_core::{Behavior, FromGeometric, GA3, IntoGeometric, behavior};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DemoState {
    pub tabs: TabsState,
    pub toggle: ToggleState,
    pub button: ButtonState,
    pub textarea: TextareaState,
    pub list: ListState,
    pub palette: CommandPaletteState,
    pub focused: Option<FocusPath>,
    pub cursor: Option<(u16, u16)>,
    pub last_input: Option<String>,
    pub bounds: Option<(u16, u16)>,
    pub status: String,
}

impl IntoGeometric for DemoState {
    fn into_geometric(self) -> GA3 {
        GA3::zero()
    }
}

impl FromGeometric for DemoState {
    fn from_geometric(_mv: &GA3) -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemoMsg {
    Tabs(TabsMsg),
    Toggle(ToggleMsg),
    Button(ButtonMsg),
    Textarea(TextareaMsg),
    List(ListMsg<()>),
    Palette(CommandPaletteMsg),
    FocusChanged(Option<FocusPath>),
    CursorChanged(Option<(u16, u16)>),
    InspectInput(String),
    InspectBounds((u16, u16)),
    OpenPalette,
}

#[derive(Debug, Clone)]
pub struct DemoContext {
    pub width: u16,
    pub height: u16,
    pub list_width: u16,
    pub tabs: TabsContext,
    pub toggle: ToggleContext,
    pub button: ButtonContext,
    pub textarea: TextareaContext,
    pub list: ListContext<String>,
    pub palette: CommandPaletteContext,
}

impl DemoContext {
    #[must_use]
    pub fn for_bounds(bounds: crate::Rect) -> Self {
        let mut ctx = Self::default();
        let workspace_width = bounds.width.saturating_sub(2).max(24);
        let workspace_height = bounds.height.saturating_sub(2).max(12);
        let body_width = workspace_width.saturating_sub(2).max(20);
        let list_width = if body_width >= 48 {
            24
        } else {
            (body_width / 3).max(14)
        };
        let textarea_width = body_width.saturating_sub(list_width).max(12);
        let editor_height = workspace_height.saturating_sub(8).clamp(4, 12);
        let palette_width = body_width.clamp(20, 40);

        ctx.width = workspace_width;
        ctx.height = workspace_height;
        ctx.list_width = list_width;
        ctx.textarea.width = textarea_width;
        ctx.textarea.height = editor_height;
        ctx.list.viewport_height = editor_height;
        ctx.palette.width = palette_width;
        ctx.palette.list_height = workspace_height.saturating_sub(10).clamp(3, 6);
        ctx.palette.input.width = palette_width.saturating_sub(2).max(10);
        ctx
    }
}

impl Default for DemoContext {
    fn default() -> Self {
        Self {
            width: 72,
            height: 20,
            list_width: 24,
            tabs: TabsContext {
                root_id: 90_000_u64.into(),
                tab_base_id: 90_100_u64.into(),
                labels: vec!["Overview".into(), "Notes".into(), "Tasks".into()],
            },
            toggle: ToggleContext {
                root_id: 91_000_u64.into(),
                box_id: 91_001_u64.into(),
                label_id: 91_002_u64.into(),
                width: 18,
                label: "shared mode".into(),
            },
            button: ButtonContext {
                root_id: 92_000_u64.into(),
                label_id: 92_001_u64.into(),
                width: 14,
                label: "sync now".into(),
            },
            textarea: TextareaContext {
                root_id: 93_000_u64.into(),
                content_id: 93_001_u64.into(),
                line_base_id: 93_100_u64.into(),
                width: 44,
                height: 8,
                placeholder: "Write shared notes...".into(),
            },
            list: ListContext {
                items: vec![
                    "agent review".into(),
                    "sync presence".into(),
                    "ship 0.1.0 demo".into(),
                ],
                viewport_height: 8,
            },
            palette: CommandPaletteContext {
                items: vec![
                    "Open Notes".into(),
                    "Toggle Shared Mode".into(),
                    "Sync Workspace".into(),
                    "Show Tasks".into(),
                ],
                width: 36,
                list_height: 5,
                input: crate::InputContext {
                    root_id: 96_000_u64.into(),
                    field_id: 96_001_u64.into(),
                    width: 34,
                    placeholder: "Type a workspace command".into(),
                },
            },
        }
    }
}

type DemoListMachine = ListMachine<String, (), fn(&String, bool) -> Scene<()>>;

#[derive(Debug, Clone)]
pub struct DemoMachine {
    tabs: TabsMachine,
    toggle: ToggleMachine,
    button: ButtonMachine,
    textarea: TextareaMachine,
    list: DemoListMachine,
    palette: CommandPaletteMachine,
}

impl Default for DemoMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl DemoMachine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tabs: TabsMachine,
            toggle: ToggleMachine,
            button: ButtonMachine,
            textarea: TextareaMachine,
            list: ListMachine::new(render_demo_item as fn(&String, bool) -> Scene<()>),
            palette: CommandPaletteMachine::new(),
        }
    }

    #[must_use]
    pub fn key_msg(
        &self,
        focus: &FocusState,
        state: &DemoState,
        event: KeyEvent,
        ctx: &DemoContext,
    ) -> Option<DemoMsg> {
        if event.ctrl && matches!(event.key, Key::Char('p') | Key::Char('P')) {
            return Some(DemoMsg::OpenPalette);
        }

        if state.palette.open {
            return self
                .palette
                .key_msg(focus, &state.palette, event, &ctx.palette)
                .map(DemoMsg::Palette);
        }

        if child_has_focus(focus, self.tabs.root_id(&ctx.tabs)) {
            self.tabs.key_msg(&state.tabs, event).map(DemoMsg::Tabs)
        } else if child_has_focus(focus, self.toggle.root_id(&ctx.toggle)) {
            self.toggle.key_msg(event).map(DemoMsg::Toggle)
        } else if child_has_focus(focus, self.button.root_id(&ctx.button)) {
            self.button.key_msg(event).map(DemoMsg::Button)
        } else if child_has_focus(focus, self.textarea.root_id(&ctx.textarea)) {
            self.textarea.key_msg(event).map(DemoMsg::Textarea)
        } else {
            dispatch_if_focused(
                focus,
                self.list.root_id(),
                self.list.key_msg(&state.list, event).map(DemoMsg::List),
            )
        }
    }

    fn apply_palette_command(&self, model: &mut DemoState, ctx: &DemoContext, index: usize) {
        match ctx.palette.items.get(index).map(String::as_str) {
            Some("Open Notes") => {
                model.tabs.selected = 1;
                model.tabs.committed = Some(1);
            }
            Some("Toggle Shared Mode") => {
                model.toggle.checked = !model.toggle.checked;
            }
            Some("Sync Workspace") => {
                model.button.activations = model.button.activations.saturating_add(1);
            }
            Some("Show Tasks") => {
                model.tabs.selected = 2;
                model.tabs.committed = Some(2);
                model.list.selected = 0;
                model.list.scroll = 0;
            }
            _ => {}
        }
    }

    fn sync_status(&self, model: &mut DemoState, ctx: &DemoContext) {
        let active_tab = ctx
            .tabs
            .labels
            .get(model.tabs.selected)
            .map_or("<none>", String::as_str);
        let committed = model
            .textarea
            .committed
            .as_deref()
            .unwrap_or("<draft>")
            .replace('\n', " | ");
        model.status = format!(
            "tab:{active_tab} shared:{} syncs:{} note:{} task:{} palette:{} commit:{:?}",
            model.toggle.checked,
            model.button.activations,
            committed,
            model.list.selected,
            model.palette.open,
            model.palette.committed,
        );
    }
}

impl Machine for DemoMachine {
    type Context = DemoContext;
    type Msg = DemoMsg;
    type Model = DemoState;
    type Shared = ();

    fn init(&self, ctx: &Self::Context) -> Self::Model {
        let mut state = DemoState {
            status: String::new(),
            palette: CommandPaletteState {
                open: false,
                ..self.palette.init(&ctx.palette)
            },
            ..DemoState::default()
        };
        self.sync_status(&mut state, ctx);
        state
    }

    fn update(
        &self,
        model: &mut Self::Model,
        msg: Self::Msg,
        ctx: &Self::Context,
    ) -> Effect<Self::Msg> {
        let effect = match msg {
            DemoMsg::Tabs(msg) => {
                update_child(&self.tabs, &mut model.tabs, msg, &ctx.tabs, &DemoMsg::Tabs)
            }
            DemoMsg::Toggle(msg) => update_child(
                &self.toggle,
                &mut model.toggle,
                msg,
                &ctx.toggle,
                &DemoMsg::Toggle,
            ),
            DemoMsg::Button(msg) => update_child(
                &self.button,
                &mut model.button,
                msg,
                &ctx.button,
                &DemoMsg::Button,
            ),
            DemoMsg::Textarea(msg) => update_child(
                &self.textarea,
                &mut model.textarea,
                msg,
                &ctx.textarea,
                &DemoMsg::Textarea,
            ),
            DemoMsg::List(msg) => {
                update_child(&self.list, &mut model.list, msg, &ctx.list, &DemoMsg::List)
            }
            DemoMsg::Palette(msg) => {
                let committed_before = model.palette.committed;
                let effect = update_child(
                    &self.palette,
                    &mut model.palette,
                    msg,
                    &ctx.palette,
                    &DemoMsg::Palette,
                );
                if let Some(index) = model.palette.committed
                    && committed_before != Some(index)
                {
                    self.apply_palette_command(model, ctx, index);
                    model.palette.open = false;
                }
                effect
            }
            DemoMsg::FocusChanged(path) => {
                model.focused = path;
                Effect::None
            }
            DemoMsg::CursorChanged(position) => {
                model.cursor = position;
                Effect::None
            }
            DemoMsg::InspectInput(input) => {
                model.last_input = Some(input);
                Effect::None
            }
            DemoMsg::InspectBounds(bounds) => {
                model.bounds = Some(bounds);
                Effect::None
            }
            DemoMsg::OpenPalette => {
                model.palette.open = true;
                model.palette.filtered = ctx
                    .palette
                    .items
                    .iter()
                    .enumerate()
                    .map(|(index, _)| index)
                    .collect();
                model.palette.input.value.clear();
                model.palette.input.cursor = 0;
                model.palette.input.committed = None;
                model.palette.list = ListState::default();
                Effect::RequestFocus(self.palette.input_root_id(&ctx.palette))
            }
        };
        self.sync_status(model, ctx);
        effect
    }

    fn project_once(
        &self,
        model: &Self::Model,
        _shared: &Self::Shared,
        ctx: &Self::Context,
    ) -> Scene<Self::Msg> {
        let mut focus = FocusState::new();
        if let Some(path) = model.focused.clone() {
            focus.set(path);
        }

        let tabs = project_child(&self.tabs, &model.tabs, &(), &ctx.tabs, &DemoMsg::Tabs)
            .with_style(if child_has_focus(&focus, self.tabs.root_id(&ctx.tabs)) {
                focus_style()
            } else {
                Style::PLAIN
            });
        let toggle = project_child(
            &self.toggle,
            &model.toggle,
            &(),
            &ctx.toggle,
            &DemoMsg::Toggle,
        )
        .with_style(
            if child_has_focus(&focus, self.toggle.root_id(&ctx.toggle)) {
                focus_style()
            } else {
                Style::PLAIN
            },
        );
        let button = project_child(
            &self.button,
            &model.button,
            &(),
            &ctx.button,
            &DemoMsg::Button,
        )
        .with_style(
            if child_has_focus(&focus, self.button.root_id(&ctx.button)) {
                focus_style()
            } else {
                Style::PLAIN
            },
        );
        let textarea = project_child(
            &self.textarea,
            &model.textarea,
            &(),
            &ctx.textarea,
            &DemoMsg::Textarea,
        )
        .with_style(
            if child_has_focus(&focus, self.textarea.root_id(&ctx.textarea)) {
                focus_style()
            } else {
                Style::PLAIN
            },
        );
        let list = project_child(&self.list, &model.list, &(), &ctx.list, &DemoMsg::List)
            .with_style(if child_has_focus(&focus, self.list.root_id()) {
                focus_style()
            } else {
                Style::PLAIN
            });
        let palette = project_child(
            &self.palette,
            &model.palette,
            &(),
            &ctx.palette,
            &DemoMsg::Palette,
        );

        let controls = Scene::row(94_000_u64, vec![toggle, button]);
        let body = Scene::row(
            94_001_u64,
            vec![
                Scene::sized(
                    94_002_u64,
                    SizeConstraint::width(ctx.textarea.width),
                    textarea,
                ),
                Scene::sized(
                    94_003_u64,
                    SizeConstraint::width(ctx.list_width),
                    Scene::border(94_004_u64, list),
                ),
            ],
        );
        let status = Scene::border(
            94_005_u64,
            Scene::column(
                94_013_u64,
                vec![
                    Scene::text(94_006_u64, model.status.clone()).with_style(Style::PLAIN.bold()),
                    Scene::text(94_014_u64, focus_summary(model, ctx)).with_style(focus_style()),
                ],
            ),
        );

        let workspace = Scene::focus_scope_with_policy(
            94_007_u64,
            "demo-root",
            FocusScopePolicy::Passthrough,
            Scene::border(
                94_008_u64,
                Scene::sized(
                    94_009_u64,
                    SizeConstraint::new(Some(ctx.width), Some(ctx.height)),
                    Scene::column(
                        94_010_u64,
                        vec![
                            Scene::text(94_011_u64, "Knopper Demo Workspace")
                                .with_role(crate::Role::Header)
                                .with_style(Style::PLAIN.bold()),
                            tabs,
                            controls,
                            body,
                            status,
                        ],
                    ),
                ),
            ),
        );

        Scene::overlay(94_012_u64, vec![workspace, palette])
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
            scene_for_model.set(DemoMachine::new().project_once(next_model, &(), &ctx));
        });
        scene
    }

    fn cursor_position(
        &self,
        model: &Self::Model,
        _shared: &Self::Shared,
        ctx: &Self::Context,
        layout: &crate::LayoutNode,
    ) -> Option<(u16, u16)> {
        let mut focus = FocusState::new();
        if let Some(path) = model.focused.clone() {
            focus.set(path);
        } else {
            return None;
        }

        if model.palette.open {
            if child_has_focus(&focus, self.palette.input_root_id(&ctx.palette)) {
                self.palette
                    .cursor_position(&model.palette, &(), &ctx.palette, layout)
            } else {
                None
            }
        } else if child_has_focus(&focus, self.textarea.root_id(&ctx.textarea)) {
            self.textarea
                .cursor_position(&model.textarea, &(), &ctx.textarea, layout)
        } else {
            None
        }
    }
}

fn focus_style() -> Style {
    Style::PLAIN.fg(Color::Ansi(6)).bold()
}

fn focus_summary(model: &DemoState, ctx: &DemoContext) -> String {
    let focus_label = match model.focused.as_ref().and_then(FocusPath::current) {
        Some(id) if id == ctx.tabs.tab_base_id || (90_100_u64..90_200_u64).contains(&id.get()) => {
            "tabs"
        }
        Some(id) if id == ctx.toggle.label_id || id == ctx.toggle.box_id => "shared-mode toggle",
        Some(id) if id == ctx.button.label_id => "sync button",
        Some(id)
            if id == ctx.textarea.root_id
                || id == ctx.textarea.content_id
                || (93_100_u64..94_000_u64).contains(&id.get()) =>
        {
            "notes editor"
        }
        Some(id) if id == ctx.palette.input.root_id || id == ctx.palette.input.field_id => {
            "palette input"
        }
        Some(id) if (20_100_u64..20_200_u64).contains(&id.get()) => "palette list",
        Some(_) if model.palette.open => "task list / modal",
        Some(_) => "task list",
        None => "none",
    };

    let bounds = model
        .bounds
        .map(|(w, h)| format!("{w}x{h}"))
        .unwrap_or_else(|| "?x?".into());
    let cursor = model
        .cursor
        .map(|(x, y)| format!("{x},{y}"))
        .unwrap_or_else(|| "hidden".into());
    format!(
        "focus:{focus_label} cursor:{cursor} bounds:{bounds} input:{}",
        model.last_input.as_deref().unwrap_or("<none>")
    )
}

fn render_demo_item(item: &String, selected: bool) -> Scene<()> {
    let content = if selected {
        format!("> {item}")
    } else {
        format!("  {item}")
    };
    Scene::text(item.len() as u64 + 95_000, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InputMsg, RenderOp, Runtime, layout::Rect};

    #[test]
    fn demo_machine_renders_workspace_sections() {
        let machine = DemoMachine::new();
        let runtime = Runtime::new(machine, DemoContext::default(), ());
        let ops = runtime.render_ops(Rect::new(0, 0, 80, 24));

        assert!(ops.iter().any(|op| matches!(op, RenderOp::DrawText { content, .. } if content == "Knopper Demo Workspace")));
        assert!(ops.iter().any(
            |op| matches!(op, RenderOp::DrawText { content, .. } if content.contains("[Overview]"))
        ));
        assert!(ops.iter().any(|op| matches!(op, RenderOp::DrawText { content, .. } if content == "Write shared notes...")));
        assert!(ops.iter().any(|op| matches!(op, RenderOp::DrawText { content, .. } if content.contains("agent review"))));
    }

    #[test]
    fn demo_machine_updates_status_from_child_machines() {
        let machine = DemoMachine::new();
        let mut runtime = Runtime::new(machine, DemoContext::default(), ());

        runtime.send(DemoMsg::Toggle(ToggleMsg::Toggle));
        runtime.send(DemoMsg::Button(ButtonMsg::Press));
        runtime.send(DemoMsg::Tabs(TabsMsg::Select(1)));
        runtime.send(DemoMsg::Textarea(TextareaMsg::Insert('h')));
        runtime.send(DemoMsg::Textarea(TextareaMsg::Commit));

        assert!(runtime.model().status.contains("tab:Notes"));
        assert!(runtime.model().status.contains("shared:true"));
        assert!(runtime.model().status.contains("syncs:1"));
        assert!(runtime.model().status.contains("note:h"));
        assert!(runtime.model().status.contains("palette:false"));
    }

    #[test]
    fn demo_machine_can_overlay_command_palette() {
        let machine = DemoMachine::new();
        let mut runtime = Runtime::new(machine, DemoContext::default(), ());

        runtime.send(DemoMsg::OpenPalette);
        runtime.send(DemoMsg::Palette(CommandPaletteMsg::Input(
            InputMsg::Insert('s'),
        )));

        let ops = runtime.render_ops(Rect::new(0, 0, 80, 24));
        assert!(runtime.model().palette.open);
        assert!(ops.iter().any(|op| matches!(op, RenderOp::DrawText { content, .. } if content.contains("committed:<none>"))));
        assert!(
            ops.iter().any(
                |op| matches!(op, RenderOp::DrawText { content, .. } if content.contains("s"))
            )
        );
    }

    #[test]
    fn palette_commit_drives_demo_actions() {
        let machine = DemoMachine::new();
        let mut runtime = Runtime::new(machine, DemoContext::default(), ());

        runtime.send(DemoMsg::OpenPalette);
        for ch in "sync".chars() {
            runtime.send(DemoMsg::Palette(CommandPaletteMsg::Input(
                InputMsg::Insert(ch),
            )));
        }
        runtime.send(DemoMsg::Palette(CommandPaletteMsg::List(ListMsg::Commit(
            0,
        ))));

        assert_eq!(runtime.model().button.activations, 1);
        assert!(!runtime.model().palette.open);

        runtime.send(DemoMsg::OpenPalette);
        for ch in "notes".chars() {
            runtime.send(DemoMsg::Palette(CommandPaletteMsg::Input(
                InputMsg::Insert(ch),
            )));
        }
        runtime.send(DemoMsg::Palette(CommandPaletteMsg::List(ListMsg::Commit(
            0,
        ))));

        assert_eq!(runtime.model().tabs.selected, 1);
        assert_eq!(runtime.model().tabs.committed, Some(1));

        runtime.send(DemoMsg::OpenPalette);
        for ch in "toggle".chars() {
            runtime.send(DemoMsg::Palette(CommandPaletteMsg::Input(
                InputMsg::Insert(ch),
            )));
        }
        runtime.send(DemoMsg::Palette(CommandPaletteMsg::List(ListMsg::Commit(
            0,
        ))));

        assert!(runtime.model().toggle.checked);
    }
}
