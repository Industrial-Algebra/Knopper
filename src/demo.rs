use crate::{
    Effect, FocusScopePolicy, Machine, Scene, SceneBehavior, SizeConstraint, Style, project_child,
    standard::{
        button::{ButtonContext, ButtonMachine, ButtonMsg, ButtonState},
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
}

#[derive(Debug, Clone)]
pub struct DemoContext {
    pub width: u16,
    pub height: u16,
    pub tabs: TabsContext,
    pub toggle: ToggleContext,
    pub button: ButtonContext,
    pub textarea: TextareaContext,
    pub list: ListContext<String>,
}

impl Default for DemoContext {
    fn default() -> Self {
        Self {
            width: 72,
            height: 20,
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
            "tab:{active_tab} shared:{} syncs:{} note:{} selected-task:{}",
            model.toggle.checked, model.button.activations, committed, model.list.selected
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
        let tabs = project_child(&self.tabs, &model.tabs, &(), &ctx.tabs, &DemoMsg::Tabs);
        let toggle = project_child(
            &self.toggle,
            &model.toggle,
            &(),
            &ctx.toggle,
            &DemoMsg::Toggle,
        );
        let button = project_child(
            &self.button,
            &model.button,
            &(),
            &ctx.button,
            &DemoMsg::Button,
        );
        let textarea = project_child(
            &self.textarea,
            &model.textarea,
            &(),
            &ctx.textarea,
            &DemoMsg::Textarea,
        );
        let list = project_child(&self.list, &model.list, &(), &ctx.list, &DemoMsg::List);

        let controls = Scene::row(94_000_u64, vec![toggle, button]);
        let body = Scene::row(
            94_001_u64,
            vec![
                Scene::sized(94_002_u64, SizeConstraint::width(46), textarea),
                Scene::sized(
                    94_003_u64,
                    SizeConstraint::width(24),
                    Scene::border(94_004_u64, list),
                ),
            ],
        );
        let status = Scene::border(
            94_005_u64,
            Scene::text(94_006_u64, model.status.clone()).with_style(Style::PLAIN.bold()),
        );

        Scene::focus_scope_with_policy(
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
        self.textarea
            .cursor_position(&model.textarea, &(), &ctx.textarea, layout)
    }
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
    use crate::{RenderOp, Runtime, layout::Rect};

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
    }
}
