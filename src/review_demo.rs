// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ButtonContext, ButtonMachine, ButtonMsg, ButtonState, Color, Effect, FocusPath,
    FocusScopePolicy, FocusState, InputContext, InputMachine, InputMsg, InputState, KeyEvent,
    ListContext, ListIds, ListMachine, ListMsg, ListState, Machine, Padding, Scene, SceneBehavior,
    SizeConstraint, Style, TabsContext, TabsMachine, TabsMsg, TabsState, TextareaContext,
    TextareaMachine, TextareaMsg, TextareaState, ToggleContext, ToggleMachine, ToggleMsg,
    ToggleState, child_has_focus,
    demo_ui::{
        PresenceCue, PresenceTone, app_shell_with_theme, bounded_surface_panel_with_theme,
        focus_style, labeled_value, master_detail, panel_theme, reading_theme, shell_theme,
        split_columns, status_theme, truncated_wrapped_lines,
    },
    dispatch_if_focused, project_child, update_child,
};
use cliffy_core::{Behavior, FromGeometric, GA3, IntoGeometric, behavior};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewPriority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewItem {
    pub title: String,
    pub author: String,
    pub summary: String,
    pub priority: ReviewPriority,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewRow {
    pub title: String,
    pub author: String,
    pub summary: String,
    pub priority: ReviewPriority,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReviewDemoState {
    pub tabs: TabsState,
    pub query: InputState,
    pub draft: TextareaState,
    pub list: ListState,
    pub follow: ToggleState,
    pub publish: ButtonState,
    pub focused: Option<FocusPath>,
    pub cursor: Option<(u16, u16)>,
    pub status: String,
}

impl IntoGeometric for ReviewDemoState {
    fn into_geometric(self) -> GA3 {
        GA3::zero()
    }
}

impl FromGeometric for ReviewDemoState {
    fn from_geometric(_mv: &GA3) -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewDemoMsg {
    Tabs(TabsMsg),
    Query(InputMsg),
    Draft(TextareaMsg),
    List(ListMsg<()>),
    Follow(ToggleMsg),
    Publish(ButtonMsg),
    FocusChanged(Option<FocusPath>),
    CursorChanged(Option<(u16, u16)>),
}

#[derive(Debug, Clone)]
pub struct ReviewDemoContext {
    pub width: u16,
    pub height: u16,
    pub list_width: u16,
    pub tabs: TabsContext,
    pub query: InputContext,
    pub draft: TextareaContext,
    pub list: ListContext<ReviewItem>,
    pub follow: ToggleContext,
    pub publish: ButtonContext,
}

impl ReviewDemoContext {
    #[must_use]
    pub fn for_bounds(bounds: crate::Rect) -> Self {
        let mut ctx = Self::default();
        let workspace_width = bounds.width.saturating_sub(2).max(30);
        let workspace_height = bounds.height.saturating_sub(2).max(14);
        let body_width = workspace_width.saturating_sub(2).max(26);
        let body_height = workspace_height.saturating_sub(8).max(8);
        let list_width = (body_width / 2).clamp(18, 32);
        let draft_width = body_width
            .saturating_sub(list_width)
            .saturating_sub(2)
            .max(18);

        ctx.width = workspace_width;
        ctx.height = workspace_height;
        ctx.list_width = list_width;
        ctx.query.width = list_width.saturating_sub(2).max(12);
        ctx.draft.width = draft_width.saturating_sub(4).max(14);
        ctx.draft.height = body_height.saturating_sub(2).clamp(6, 12);
        ctx.list.viewport_height = body_height.saturating_sub(6).clamp(4, 8);
        ctx
    }
}

impl Default for ReviewDemoContext {
    fn default() -> Self {
        Self {
            width: 76,
            height: 22,
            list_width: 28,
            tabs: TabsContext {
                root_id: 110_000_u64.into(),
                tab_base_id: 110_100_u64.into(),
                labels: vec!["Inbox".into(), "Drafts".into(), "Published".into()],
            },
            query: InputContext {
                root_id: 111_000_u64.into(),
                field_id: 111_001_u64.into(),
                width: 24,
                placeholder: "Filter reviews".into(),
            },
            draft: TextareaContext {
                root_id: 112_000_u64.into(),
                content_id: 112_001_u64.into(),
                line_base_id: 112_100_u64.into(),
                width: 34,
                height: 8,
                placeholder: "Write reviewer notes...".into(),
            },
            list: ListContext {
                items: vec![
                    ReviewItem {
                        title: "focus policy follow-up".into(),
                        author: "mika".into(),
                        summary: "Clarify passthrough vs trap semantics before the next API cut."
                            .into(),
                        priority: ReviewPriority::High,
                    },
                    ReviewItem {
                        title: "notcurses backend audit".into(),
                        author: "ava".into(),
                        summary: "Check plane lifecycle and clear-rect behavior across modal transitions."
                            .into(),
                        priority: ReviewPriority::Medium,
                    },
                    ReviewItem {
                        title: "guide copy pass".into(),
                        author: "rhea".into(),
                        summary: "Tighten the machine authoring guide before 0.1.0 preview."
                            .into(),
                        priority: ReviewPriority::Low,
                    },
                ],
                viewport_height: 6,
            },
            follow: ToggleContext {
                root_id: 113_000_u64.into(),
                box_id: 113_001_u64.into(),
                label_id: 113_002_u64.into(),
                width: 18,
                label: "Follow thread".into(),
            },
            publish: ButtonContext {
                root_id: 114_000_u64.into(),
                label_id: 114_001_u64.into(),
                width: 14,
                label: "Publish note".into(),
            },
        }
    }
}

type ReviewListMachine = ListMachine<ReviewRow, (), fn(&ReviewRow, bool) -> Scene<()>>;

#[derive(Debug, Clone)]
pub struct ReviewDemoMachine {
    tabs: TabsMachine,
    query: InputMachine,
    draft: TextareaMachine,
    list: ReviewListMachine,
    follow: ToggleMachine,
    publish: ButtonMachine,
}

impl Default for ReviewDemoMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl ReviewDemoMachine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tabs: TabsMachine,
            query: InputMachine,
            draft: TextareaMachine,
            list: ListMachine::new(render_review_row as fn(&ReviewRow, bool) -> Scene<()>)
                .with_ids(ListIds {
                    root: 115_000_u64.into(),
                    viewport: 115_001_u64.into(),
                    scroll: 115_002_u64.into(),
                    column: 115_003_u64.into(),
                    empty: 115_004_u64.into(),
                    item_base: 115_100_u64.into(),
                    marker_base: 115_200_u64.into(),
                }),
            follow: ToggleMachine,
            publish: ButtonMachine,
        }
    }

    #[must_use]
    pub fn key_msg(
        &self,
        focus: &FocusState,
        state: &ReviewDemoState,
        event: KeyEvent,
        ctx: &ReviewDemoContext,
    ) -> Option<ReviewDemoMsg> {
        if child_has_focus(focus, self.tabs.root_id(&ctx.tabs)) {
            self.tabs
                .key_msg(&state.tabs, event)
                .map(ReviewDemoMsg::Tabs)
        } else if child_has_focus(focus, self.query.root_id(&ctx.query)) {
            self.query.key_msg(event).map(ReviewDemoMsg::Query)
        } else if child_has_focus(focus, self.follow.root_id(&ctx.follow)) {
            self.follow.key_msg(event).map(ReviewDemoMsg::Follow)
        } else if child_has_focus(focus, self.publish.root_id(&ctx.publish)) {
            self.publish.key_msg(event).map(ReviewDemoMsg::Publish)
        } else if child_has_focus(focus, self.draft.root_id(&ctx.draft)) {
            self.draft.key_msg(event).map(ReviewDemoMsg::Draft)
        } else {
            dispatch_if_focused(
                focus,
                self.list.root_id(),
                self.list
                    .key_msg(&state.list, event)
                    .map(ReviewDemoMsg::List),
            )
        }
    }

    fn filtered_items(&self, model: &ReviewDemoState, ctx: &ReviewDemoContext) -> Vec<ReviewItem> {
        let query = model.query.value.trim().to_ascii_lowercase();
        ctx.list
            .items
            .iter()
            .filter(|item| {
                query.is_empty()
                    || item.title.to_ascii_lowercase().contains(&query)
                    || item.author.to_ascii_lowercase().contains(&query)
                    || item.summary.to_ascii_lowercase().contains(&query)
            })
            .cloned()
            .collect()
    }

    fn list_context(
        &self,
        model: &ReviewDemoState,
        ctx: &ReviewDemoContext,
    ) -> ListContext<ReviewRow> {
        ListContext {
            items: self
                .filtered_items(model, ctx)
                .into_iter()
                .map(|item| ReviewRow {
                    title: item.title,
                    author: item.author,
                    summary: item.summary,
                    priority: item.priority,
                })
                .collect(),
            viewport_height: ctx.list.viewport_height,
        }
    }

    fn sync_status(&self, model: &mut ReviewDemoState, ctx: &ReviewDemoContext) {
        let tab = ctx
            .tabs
            .labels
            .get(model.tabs.selected)
            .map_or("<none>", String::as_str);
        let visible = self.filtered_items(model, ctx).len();
        model.status = format!(
            "{tab} · visible reviews {visible} · following {} · published {}",
            model.follow.checked, model.publish.activations,
        );
    }
}

impl Machine for ReviewDemoMachine {
    type Context = ReviewDemoContext;
    type Msg = ReviewDemoMsg;
    type Model = ReviewDemoState;
    type Shared = ();

    fn init(&self, ctx: &Self::Context) -> Self::Model {
        let mut state = ReviewDemoState::default();
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
            ReviewDemoMsg::Tabs(msg) => update_child(
                &self.tabs,
                &mut model.tabs,
                msg,
                &ctx.tabs,
                &ReviewDemoMsg::Tabs,
            ),
            ReviewDemoMsg::Query(msg) => {
                let effect = update_child(
                    &self.query,
                    &mut model.query,
                    msg,
                    &ctx.query,
                    &ReviewDemoMsg::Query,
                );
                model.list.selected = 0;
                model.list.scroll = 0;
                effect
            }
            ReviewDemoMsg::Draft(msg) => update_child(
                &self.draft,
                &mut model.draft,
                msg,
                &ctx.draft,
                &ReviewDemoMsg::Draft,
            ),
            ReviewDemoMsg::List(msg) => {
                let list_ctx = self.list_context(model, ctx);
                update_child(
                    &self.list,
                    &mut model.list,
                    msg,
                    &list_ctx,
                    &ReviewDemoMsg::List,
                )
            }
            ReviewDemoMsg::Follow(msg) => update_child(
                &self.follow,
                &mut model.follow,
                msg,
                &ctx.follow,
                &ReviewDemoMsg::Follow,
            ),
            ReviewDemoMsg::Publish(msg) => {
                let before = model.publish.activations;
                let effect = update_child(
                    &self.publish,
                    &mut model.publish,
                    msg,
                    &ctx.publish,
                    &ReviewDemoMsg::Publish,
                );
                if model.publish.activations != before {
                    model.draft.committed = Some(model.draft.value.clone());
                }
                effect
            }
            ReviewDemoMsg::FocusChanged(path) => {
                model.focused = path;
                Effect::None
            }
            ReviewDemoMsg::CursorChanged(position) => {
                model.cursor = position;
                Effect::None
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

        let tabs = project_child(
            &self.tabs,
            &model.tabs,
            &(),
            &ctx.tabs,
            &ReviewDemoMsg::Tabs,
        )
        .with_style(if child_has_focus(&focus, self.tabs.root_id(&ctx.tabs)) {
            focus_style()
        } else {
            Style::PLAIN
        });
        let query = project_child(
            &self.query,
            &model.query,
            &(),
            &ctx.query,
            &ReviewDemoMsg::Query,
        )
        .with_style(if child_has_focus(&focus, self.query.root_id(&ctx.query)) {
            focus_style()
        } else {
            Style::PLAIN
        });
        let follow = project_child(
            &self.follow,
            &model.follow,
            &(),
            &ctx.follow,
            &ReviewDemoMsg::Follow,
        )
        .with_style(
            if child_has_focus(&focus, self.follow.root_id(&ctx.follow)) {
                focus_style()
            } else {
                Style::PLAIN
            },
        );
        let publish = project_child(
            &self.publish,
            &model.publish,
            &(),
            &ctx.publish,
            &ReviewDemoMsg::Publish,
        )
        .with_style(
            if child_has_focus(&focus, self.publish.root_id(&ctx.publish)) {
                focus_style()
            } else {
                Style::PLAIN
            },
        );
        let draft = project_child(
            &self.draft,
            &model.draft,
            &(),
            &ctx.draft,
            &ReviewDemoMsg::Draft,
        )
        .with_style(if child_has_focus(&focus, self.draft.root_id(&ctx.draft)) {
            focus_style()
        } else {
            Style::PLAIN
        });
        let list_ctx = self.list_context(model, ctx);
        let list = project_child(
            &self.list,
            &model.list,
            &(),
            &list_ctx,
            &ReviewDemoMsg::List,
        )
        .with_style(if child_has_focus(&focus, self.list.root_id()) {
            focus_style()
        } else {
            Style::PLAIN
        });

        let selected = list_ctx.items.get(model.list.selected);
        let panel_body_height = ctx.draft.height.saturating_add(2).max(9);
        let left_body = Scene::column(
            116_000_u64,
            vec![
                query,
                master_detail(
                    116_001_u64,
                    ctx.list_width.saturating_sub(4),
                    ctx.list.viewport_height.saturating_add(2),
                    panel_body_height
                        .saturating_sub(ctx.list.viewport_height)
                        .saturating_sub(3)
                        .max(4),
                    Scene::border(116_010_u64, list).with_style(Style::PLAIN.fg(Color::Ansi(8))),
                    Scene::border(
                        116_002_u64,
                        Scene::sized(
                            116_003_u64,
                            SizeConstraint::width(ctx.list_width.saturating_sub(8)),
                            review_detail_scene(selected, ctx.list_width.saturating_sub(14)),
                        ),
                    )
                    .with_style(reading_theme().frame),
                ),
            ],
        );
        let right_body = Scene::column(
            116_010_u64,
            vec![
                Scene::row(
                    116_011_u64,
                    vec![follow, Scene::text(116_012_u64, "  "), publish],
                )
                .with_style(Style::PLAIN.bg(Color::Ansi(0))),
                draft,
            ],
        )
        .with_style(Style::PLAIN.bg(Color::Ansi(0)));

        let layout = split_columns(
            116_020_u64,
            ctx.list_width,
            ctx.draft.width.saturating_add(6),
            bounded_surface_panel_with_theme(
                116_022_u64,
                ctx.list_width.saturating_sub(4),
                panel_body_height,
                "Review Queue",
                "Filter and inspect pending reviews",
                &review_presence(selected),
                left_body,
                panel_theme(
                    child_has_focus(&focus, self.list.root_id())
                        || child_has_focus(&focus, self.query.root_id(&ctx.query)),
                ),
            ),
            bounded_surface_panel_with_theme(
                116_025_u64,
                ctx.draft.width.saturating_add(2),
                panel_body_height,
                "Draft Reply",
                "Compose participant-local review notes",
                &draft_presence(model.follow.checked),
                right_body,
                panel_theme(
                    child_has_focus(&focus, self.draft.root_id(&ctx.draft))
                        || child_has_focus(&focus, self.follow.root_id(&ctx.follow))
                        || child_has_focus(&focus, self.publish.root_id(&ctx.publish)),
                ),
            ),
        );

        Scene::focus_scope_with_policy(
            116_030_u64,
            "review-demo-root",
            FocusScopePolicy::Passthrough,
            app_shell_with_theme(
                116_031_u64,
                ctx.width,
                ctx.height,
                "Knopper Review Demo",
                tabs,
                Scene::padding(116_035_u64, Padding::all(1), layout),
                Scene::padding(
                    116_036_u64,
                    Padding::all(1),
                    Scene::border(
                        116_037_u64,
                        Scene::text(116_038_u64, model.status.clone())
                            .with_style(Style::PLAIN.bold()),
                    )
                    .with_style(status_theme().frame),
                ),
                shell_theme(),
            ),
        )
    }

    fn project(
        &self,
        model: Behavior<Self::Model>,
        _shared: Behavior<Self::Shared>,
        ctx: &Self::Context,
    ) -> SceneBehavior<Self::Msg> {
        let scene = behavior(self.project_once(&model.sample(), &(), ctx));
        let ctx = ctx.clone();
        let scene_for_model = scene.clone();
        model.subscribe(move |next_model| {
            scene_for_model.set(ReviewDemoMachine::new().project_once(next_model, &(), &ctx));
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
        let path = model.focused.clone()?;
        focus.set(path);

        if child_has_focus(&focus, self.query.root_id(&ctx.query)) {
            self.query
                .cursor_position(&model.query, &(), &ctx.query, layout)
        } else if child_has_focus(&focus, self.draft.root_id(&ctx.draft)) {
            self.draft
                .cursor_position(&model.draft, &(), &ctx.draft, layout)
        } else {
            None
        }
    }
}

fn review_presence(selected: Option<&ReviewRow>) -> Vec<PresenceCue> {
    vec![
        PresenceCue {
            name: "you",
            label: "triaging",
            tone: PresenceTone::Local,
        },
        PresenceCue {
            name: "mika",
            label: selected.map_or("queued", |_| "watching"),
            tone: PresenceTone::Collaborator,
        },
    ]
}

fn draft_presence(following: bool) -> Vec<PresenceCue> {
    vec![
        PresenceCue {
            name: "you",
            label: "drafting",
            tone: PresenceTone::Local,
        },
        PresenceCue {
            name: "rhea",
            label: if following { "following" } else { "idle" },
            tone: PresenceTone::Passive,
        },
    ]
}

fn review_detail_scene(item: Option<&ReviewRow>, width: u16) -> Scene<ReviewDemoMsg> {
    match item {
        Some(item) => Scene::column(
            117_000_u64,
            vec![
                Scene::text(117_001_u64, "Selected Review")
                    .with_style(Style::PLAIN.fg(Color::Ansi(7)).bold()),
                labeled_value(
                    117_002_u64,
                    "author",
                    item.author.clone(),
                    Style::PLAIN.fg(Color::Ansi(6)).bold(),
                ),
                labeled_value(
                    117_003_u64,
                    "priority",
                    match item.priority {
                        ReviewPriority::High => "high".into(),
                        ReviewPriority::Medium => "medium".into(),
                        ReviewPriority::Low => "low".into(),
                    },
                    review_priority_style(&item.priority),
                ),
                Scene::column(
                    117_004_u64,
                    std::iter::once(
                        Scene::text(117_005_u64, "summary:")
                            .with_style(Style::PLAIN.fg(Color::Ansi(8)).bold()),
                    )
                    .chain(
                        truncated_wrapped_lines(&item.summary, width, 3)
                            .into_iter()
                            .enumerate()
                            .map(|(index, line)| {
                                Scene::text(
                                    117_006_u64 + u64::try_from(index).unwrap_or(0),
                                    format!("  {line}"),
                                )
                                .with_style(Style::PLAIN.fg(Color::Ansi(8)))
                            }),
                    )
                    .collect::<Vec<_>>(),
                ),
            ],
        )
        .with_style(Style::PLAIN.bg(Color::Ansi(0))),
        None => Scene::text(117_100_u64, "No review selected")
            .with_style(Style::PLAIN.fg(Color::Ansi(8)).bg(Color::Ansi(0))),
    }
}

fn render_review_row(item: &ReviewRow, selected: bool) -> Scene<()> {
    let base = 118_000_u64 + u64::try_from(item.title.len()).unwrap_or(0) * 10;
    Scene::column(
        base,
        vec![
            Scene::row(
                base + 1,
                vec![
                    Scene::text(base + 2, item.title.clone()).with_style(if selected {
                        Style::PLAIN.fg(Color::Ansi(3)).bold()
                    } else {
                        Style::PLAIN.fg(Color::Ansi(7)).bold()
                    }),
                    Scene::text(base + 3, "  "),
                    Scene::text(base + 4, format!("@{}", item.author))
                        .with_style(Style::PLAIN.fg(Color::Ansi(8))),
                ],
            ),
            Scene::text(base + 5, format!("  {}", item.summary))
                .with_style(Style::PLAIN.fg(Color::Ansi(8))),
        ],
    )
}

fn review_priority_style(priority: &ReviewPriority) -> Style {
    match priority {
        ReviewPriority::High => Style::PLAIN.fg(Color::Ansi(1)).bold(),
        ReviewPriority::Medium => Style::PLAIN.fg(Color::Ansi(3)).bold(),
        ReviewPriority::Low => Style::PLAIN.fg(Color::Ansi(6)).bold(),
    }
}
