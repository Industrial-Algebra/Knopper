use crate::{
    Color, Effect, FocusPath, FocusScopePolicy, FocusState, Key, KeyEvent, Machine, Padding, Scene,
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
    pub task_done: Vec<bool>,
    pub palette: CommandPaletteState,
    pub focused: Option<FocusPath>,
    pub cursor: Option<(u16, u16)>,
    pub last_input: Option<String>,
    pub bounds: Option<(u16, u16)>,
    pub inspector_visible: bool,
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
    ToggleInspector,
    OpenPalette,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskPriority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskItem {
    pub title: String,
    pub detail: String,
    pub priority: TaskPriority,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRow {
    pub title: String,
    pub detail: String,
    pub priority: TaskPriority,
    pub done: bool,
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
    pub list: ListContext<TaskItem>,
    pub palette: CommandPaletteContext,
}

impl DemoContext {
    #[must_use]
    pub fn for_bounds(bounds: crate::Rect) -> Self {
        let mut ctx = Self::default();
        let workspace_width = bounds.width.saturating_sub(2).max(24);
        let workspace_height = bounds.height.saturating_sub(2).max(12);
        let body_width = workspace_width.saturating_sub(2).max(20);
        let gutter = 2;
        let list_width = if body_width >= 64 {
            (body_width / 3).clamp(24, 30)
        } else if body_width >= 48 {
            22
        } else {
            (body_width / 3).max(14)
        };
        let textarea_width = body_width
            .saturating_sub(list_width)
            .saturating_sub(gutter)
            .max(12);
        let editor_height = workspace_height.saturating_sub(10).clamp(5, 14);
        let palette_width = body_width.clamp(28, 52);

        ctx.width = workspace_width;
        ctx.height = workspace_height;
        ctx.list_width = list_width;
        ctx.textarea.width = textarea_width;
        ctx.textarea.height = editor_height;
        ctx.list.viewport_height = editor_height;
        ctx.palette.width = palette_width;
        ctx.palette.height = workspace_height.saturating_sub(4).clamp(10, 14);
        ctx.palette.list_height = workspace_height.saturating_sub(12).clamp(4, 8);
        ctx.palette.input.width = palette_width.saturating_sub(4).max(10);
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
                label: "Shared mode".into(),
            },
            button: ButtonContext {
                root_id: 92_000_u64.into(),
                label_id: 92_001_u64.into(),
                width: 14,
                label: "Sync workspace".into(),
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
                    TaskItem {
                        title: "agent review".into(),
                        detail: "Check notes and confirm machine boundaries".into(),
                        priority: TaskPriority::High,
                    },
                    TaskItem {
                        title: "sync presence".into(),
                        detail: "Align local presence overlays with shared state seams".into(),
                        priority: TaskPriority::Medium,
                    },
                    TaskItem {
                        title: "ship 0.1.0 demo".into(),
                        detail: "Polish the interactive workspace for the first release".into(),
                        priority: TaskPriority::Low,
                    },
                ],
                viewport_height: 8,
            },
            palette: CommandPaletteContext {
                items: vec![
                    "Open Overview".into(),
                    "Open Notes".into(),
                    "Show Tasks".into(),
                    "Toggle Shared Mode".into(),
                    "Sync Workspace".into(),
                    "Draft Sync Note".into(),
                    "Toggle Selected Task".into(),
                ],
                width: 36,
                height: 10,
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

type DemoListMachine = ListMachine<TaskRow, (), fn(&TaskRow, bool) -> Scene<()>>;

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
            list: ListMachine::new(render_demo_item as fn(&TaskRow, bool) -> Scene<()>),
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

    fn task_list_context(&self, model: &DemoState, ctx: &DemoContext) -> ListContext<TaskRow> {
        ListContext {
            items: ctx
                .list
                .items
                .iter()
                .enumerate()
                .map(|(index, item)| TaskRow {
                    title: item.title.clone(),
                    detail: item.detail.clone(),
                    priority: item.priority.clone(),
                    done: model.task_done.get(index).copied().unwrap_or(false),
                })
                .collect(),
            viewport_height: ctx.list.viewport_height,
        }
    }

    fn toggle_selected_task(&self, model: &mut DemoState, ctx: &DemoContext, index: usize) {
        if index >= ctx.list.items.len() {
            return;
        }
        if model.task_done.len() < ctx.list.items.len() {
            model.task_done.resize(ctx.list.items.len(), false);
        }
        if let Some(done) = model.task_done.get_mut(index) {
            *done = !*done;
        }
    }

    fn apply_palette_command(&self, model: &mut DemoState, ctx: &DemoContext, index: usize) {
        match ctx.palette.items.get(index).map(String::as_str) {
            Some("Open Overview") => {
                model.tabs.selected = 0;
                model.tabs.committed = Some(0);
            }
            Some("Open Notes") => {
                model.tabs.selected = 1;
                model.tabs.committed = Some(1);
            }
            Some("Show Tasks") => {
                model.tabs.selected = 2;
                model.tabs.committed = Some(2);
                model.list.selected = 0;
                model.list.scroll = 0;
            }
            Some("Toggle Selected Task") => {
                self.toggle_selected_task(model, ctx, model.list.selected);
            }
            Some("Toggle Shared Mode") => {
                model.toggle.checked = !model.toggle.checked;
            }
            Some("Sync Workspace") => {
                model.button.activations = model.button.activations.saturating_add(1);
                model.textarea.committed = Some(model.textarea.value.clone());
            }
            Some("Draft Sync Note") => {
                model.tabs.selected = 1;
                model.tabs.committed = Some(1);
                model.textarea.value =
                    "Synced workspace updates:\n- review notes\n- confirm tasks\n- share status"
                        .into();
                model.textarea.cursor_row = 2;
                model.textarea.cursor_col = 14;
                model.textarea.preferred_col = Some(14);
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
        let active_task = ctx.list.items.get(model.list.selected).map_or_else(
            || "none".to_string(),
            |item| {
                let done = model
                    .task_done
                    .get(model.list.selected)
                    .copied()
                    .unwrap_or(false);
                let priority = match item.priority {
                    TaskPriority::High => "high",
                    TaskPriority::Medium => "med",
                    TaskPriority::Low => "low",
                };
                format!(
                    "{}{} ({priority})",
                    if done { "done " } else { "" },
                    item.title
                )
            },
        );
        let note_state = match model.textarea.committed.as_deref() {
            Some(value) if value == model.textarea.value => {
                let compact = value.replace('\n', " | ");
                if compact.is_empty() {
                    "empty".to_string()
                } else {
                    format!("saved:{compact}")
                }
            }
            Some(_) | None if model.textarea.value.is_empty() => "empty".to_string(),
            Some(_) => "draft edits".to_string(),
            None => "draft".to_string(),
        };
        let shared_mode = if model.toggle.checked {
            "shared"
        } else {
            "local"
        };
        model.status = format!(
            "{active_tab} · {shared_mode} · syncs {} · notes {note_state} · task {active_task}{}",
            model.button.activations,
            if model.palette.open {
                " · quick actions"
            } else {
                ""
            },
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
            task_done: vec![false; ctx.list.items.len()],
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
            DemoMsg::Button(msg) => {
                let activations_before = model.button.activations;
                let effect = update_child(
                    &self.button,
                    &mut model.button,
                    msg,
                    &ctx.button,
                    &DemoMsg::Button,
                );
                if model.button.activations != activations_before {
                    model.textarea.committed = Some(model.textarea.value.clone());
                }
                effect
            }
            DemoMsg::Textarea(msg) => update_child(
                &self.textarea,
                &mut model.textarea,
                msg,
                &ctx.textarea,
                &DemoMsg::Textarea,
            ),
            DemoMsg::List(msg) => {
                if let ListMsg::Commit(index) = msg {
                    self.toggle_selected_task(model, ctx, index);
                }
                let list_ctx = self.task_list_context(model, ctx);
                update_child(&self.list, &mut model.list, msg, &list_ctx, &DemoMsg::List)
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
            DemoMsg::ToggleInspector => {
                model.inspector_visible = !model.inspector_visible;
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

        let tabs_focused = child_has_focus(&focus, self.tabs.root_id(&ctx.tabs));
        let toggle_focused = child_has_focus(&focus, self.toggle.root_id(&ctx.toggle));
        let button_focused = child_has_focus(&focus, self.button.root_id(&ctx.button));
        let notes_focused = child_has_focus(&focus, self.textarea.root_id(&ctx.textarea));
        let tasks_focused = child_has_focus(&focus, self.list.root_id());

        let tabs = project_child(&self.tabs, &model.tabs, &(), &ctx.tabs, &DemoMsg::Tabs)
            .with_style(if tabs_focused {
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
        .with_style(if toggle_focused {
            focus_style()
        } else {
            Style::PLAIN
        });
        let button = project_child(
            &self.button,
            &model.button,
            &(),
            &ctx.button,
            &DemoMsg::Button,
        )
        .with_style(if button_focused {
            focus_style()
        } else {
            Style::PLAIN
        });
        let textarea = project_child(
            &self.textarea,
            &model.textarea,
            &(),
            &ctx.textarea,
            &DemoMsg::Textarea,
        )
        .with_style(if notes_focused {
            focus_style()
        } else {
            Style::PLAIN
        });
        let list_ctx = self.task_list_context(model, ctx);
        let list = project_child(&self.list, &model.list, &(), &list_ctx, &DemoMsg::List)
            .with_style(if tasks_focused {
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

        let controls = Scene::padding(
            94_000_u64,
            Padding {
                top: 1,
                right: 0,
                bottom: 0,
                left: 0,
            },
            Scene::row(
                94_015_u64,
                vec![toggle, Scene::text(94_016_u64, "  "), button],
            ),
        );
        let notes_pane = Scene::border(
            94_020_u64,
            Scene::padding(
                94_024_u64,
                Padding::all(1),
                Scene::column(
                    94_025_u64,
                    vec![
                        Scene::text(94_021_u64, "Notes")
                            .with_style(section_title_style(notes_focused)),
                        Scene::text(
                            94_026_u64,
                            if notes_focused {
                                "Editing shared draft"
                            } else {
                                "Shared notes surface"
                            },
                        )
                        .with_style(Style::PLAIN.fg(Color::Ansi(8))),
                        textarea,
                    ],
                ),
            ),
        )
        .with_style(surface_style(notes_focused));
        let tasks_pane = Scene::border(
            94_022_u64,
            Scene::padding(
                94_027_u64,
                Padding::all(1),
                Scene::column(
                    94_028_u64,
                    vec![
                        Scene::text(94_023_u64, "Tasks")
                            .with_style(section_title_style(tasks_focused)),
                        Scene::text(
                            94_029_u64,
                            if tasks_focused {
                                "Navigate and commit tasks"
                            } else {
                                "Focus to triage work"
                            },
                        )
                        .with_style(Style::PLAIN.fg(Color::Ansi(8))),
                        Scene::border(94_004_u64, list).with_style(if tasks_focused {
                            focus_style()
                        } else {
                            Style::PLAIN.fg(Color::Ansi(8))
                        }),
                    ],
                ),
            ),
        )
        .with_style(surface_style(tasks_focused));
        let body = Scene::padding(
            94_001_u64,
            Padding {
                top: 1,
                right: 0,
                bottom: 0,
                left: 0,
            },
            Scene::row(
                94_017_u64,
                vec![
                    Scene::sized(
                        94_002_u64,
                        SizeConstraint::width(ctx.textarea.width),
                        notes_pane,
                    ),
                    Scene::text(94_018_u64, "  "),
                    Scene::sized(
                        94_003_u64,
                        SizeConstraint::width(ctx.list_width),
                        tasks_pane,
                    ),
                ],
            ),
        );
        let mut status_children =
            vec![Scene::text(94_006_u64, model.status.clone()).with_style(Style::PLAIN.bold())];
        if model.inspector_visible {
            status_children
                .push(Scene::text(94_014_u64, focus_summary(model, ctx)).with_style(focus_style()));
        }
        let status = Scene::padding(
            94_019_u64,
            Padding {
                top: 1,
                right: 0,
                bottom: 0,
                left: 0,
            },
            Scene::border(94_005_u64, Scene::column(94_013_u64, status_children))
                .with_style(Style::PLAIN.fg(Color::Ansi(8))),
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
                                .with_style(Style::PLAIN.fg(Color::Ansi(6)).bold()),
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

fn surface_style(active: bool) -> Style {
    if active {
        Style::PLAIN.fg(Color::Ansi(6)).bold()
    } else {
        Style::PLAIN.fg(Color::Ansi(8))
    }
}

fn section_title_style(active: bool) -> Style {
    if active {
        Style::PLAIN.fg(Color::Ansi(6)).bold()
    } else {
        Style::PLAIN.fg(Color::Ansi(7)).bold()
    }
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

fn render_demo_item(item: &TaskRow, selected: bool) -> Scene<()> {
    let status = if item.done { "[x]" } else { "[ ]" };
    let priority = match item.priority {
        TaskPriority::High => "high",
        TaskPriority::Medium => "med",
        TaskPriority::Low => "low",
    };
    let content = if selected {
        format!("> {status} {} · {priority} · {}", item.title, item.detail)
    } else {
        format!("  {status} {} · {priority} · {}", item.title, item.detail)
    };
    Scene::text(item.title.len() as u64 + 95_000, content).with_style(if item.done {
        Style::PLAIN.fg(Color::Ansi(2)).bold()
    } else {
        match item.priority {
            TaskPriority::High => Style::PLAIN.fg(Color::Ansi(1)).bold(),
            TaskPriority::Medium => Style::PLAIN.fg(Color::Ansi(3)).bold(),
            TaskPriority::Low => Style::PLAIN.fg(Color::Ansi(6)),
        }
    })
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

        assert!(runtime.model().status.contains("Notes"));
        assert!(runtime.model().status.contains("shared"));
        assert!(runtime.model().status.contains("syncs 1"));
        assert!(runtime.model().status.contains("saved:h"));
        assert!(!runtime.model().status.contains("palette"));
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
        assert!(ops.iter().any(|op| matches!(op, RenderOp::DrawText { content, .. } if content.contains("last action: none"))));
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
