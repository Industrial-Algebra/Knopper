pub mod annotation;
pub mod backend;
pub mod collaboration;
pub mod compose;
pub mod demo;
pub mod demo_ui;
pub mod diff;
pub mod effect;
pub mod focus;
pub mod id;
pub mod input;
pub mod layout;
pub mod machine;
pub mod render;
pub mod renderer;
pub mod review_demo;
pub mod routing;
pub mod runtime;
pub mod scene;
pub mod standard;
pub mod style;

pub use annotation::Annotation;
#[cfg(feature = "notcurses")]
pub use backend::notcurses::NotcursesBackend;
pub use backend::{
    BackendCommand, BackendEntry, BackendState, MockBackend, TerminalBackend, backend_commands,
};
pub use collaboration::{ParticipantId, ParticipantRoster, Presence, PresenceTone};
pub use compose::{
    child_has_focus, dispatch_if_focused, map_effect, project_child, trap_focus, update_child,
};
pub use demo::{DemoContext, DemoMachine, DemoMsg, DemoState};
pub use diff::{PatchOp, diff_render_ops};
pub use effect::Effect;
pub use focus::{FocusNavigation, FocusOrder, FocusPath, FocusState};
pub use id::NodeId;
pub use input::{Key, KeyEvent, ResizeEvent, RuntimeEvent};
pub use layout::{LayoutKind, LayoutNode, Rect, Size, find_node, measure, resolve_layout};
pub use machine::{Machine, PureMachine, SceneBehavior};
pub use render::{RenderOp, render_ops};
pub use renderer::{MockRenderer, Renderer, render_once};
pub use review_demo::{ReviewDemoContext, ReviewDemoMachine, ReviewDemoMsg, ReviewDemoState};
pub use routing::{RoutedEvent, activation_message, focus_path, route_event};
pub use runtime::Runtime;
pub use scene::{
    Anchor, FocusScopePolicy, HorizontalAlign, Interaction, NodeMeta, Padding, Role, Scene,
    ScrollOffset, SizeConstraint, TextNode, VerticalAlign,
};
pub use standard::button::{ButtonContext, ButtonMachine, ButtonMsg, ButtonState, button_key_msg};
pub use standard::command_palette::{
    CommandPaletteContext, CommandPaletteMachine, CommandPaletteMsg, CommandPaletteState,
};
pub use standard::input::{InputContext, InputMachine, InputMsg, InputState, input_key_msg};
pub use standard::list::{ListContext, ListIds, ListMachine, ListMsg, ListState, list_key_msg};
pub use standard::list_detail::{
    ListDetailContext, ListDetailMachine, ListDetailMsg, ListDetailState,
};
pub use standard::modal::{
    ModalFocusConfig, ModalIds, ModalKeyAction, ModalMsg, modal_key_msg, modal_scene,
};
pub use standard::tabs::{TabsContext, TabsMachine, TabsMsg, TabsState, tabs_key_msg};
pub use standard::textarea::{
    TextareaContext, TextareaMachine, TextareaMsg, TextareaState, textarea_key_msg,
};
pub use standard::toggle::{ToggleContext, ToggleMachine, ToggleMsg, ToggleState, toggle_key_msg};
pub use style::{Color, Emphasis, Style};
