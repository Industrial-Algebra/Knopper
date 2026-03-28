pub mod annotation;
pub mod backend;
pub mod diff;
pub mod effect;
pub mod focus;
pub mod id;
pub mod input;
pub mod layout;
pub mod machine;
pub mod render;
pub mod renderer;
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
pub use diff::{PatchOp, diff_render_ops};
pub use effect::Effect;
pub use focus::{FocusPath, FocusState};
pub use id::NodeId;
pub use input::{Key, KeyEvent, ResizeEvent, RuntimeEvent};
pub use layout::{LayoutKind, LayoutNode, Rect, Size, measure, resolve_layout};
pub use machine::{Machine, PureMachine, SceneBehavior};
pub use render::{RenderOp, render_ops};
pub use renderer::{MockRenderer, Renderer, render_once};
pub use routing::{RoutedEvent, activation_message, focus_path, route_event};
pub use runtime::Runtime;
pub use scene::{
    Interaction, NodeMeta, Padding, Role, Scene, ScrollOffset, SizeConstraint, TextNode,
};
pub use standard::list::{ListContext, ListIds, ListMachine, ListMsg, ListState};
pub use style::{Color, Emphasis, Style};
