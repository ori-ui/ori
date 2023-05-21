mod build;
mod children;
mod context;
mod debug;
mod event;
mod layout;
mod node;
mod root_node;
mod style;
mod unit;
mod view;
mod views;

pub use build::*;
pub use children::*;
pub use context::*;
pub use debug::*;
pub use event::*;
pub use layout::*;
pub use node::*;
pub use root_node::*;
pub use style::*;
pub use unit::*;
pub use view::*;
pub use views::*;

pub use glam::*;
pub use tracing::{debug, error, info, trace, warn};

pub mod prelude {
    pub use crate::children::{Children, FlexLayout};
    pub use crate::context::{Context, DrawContext, EventContext, LayoutContext};
    pub use crate::event::{
        Cursor, Key, KeyboardEvent, Modifiers, PointerButton, PointerEvent, RequestRedrawEvent,
        SetWindowIconEvent, SetWindowTitleEvent, WindowResizeEvent,
    };
    pub use crate::layout::{AlignItem, AvailableSpace, Axis, JustifyContent, Margin, Padding};
    pub use crate::node::Node;
    pub use crate::style::Styleable;
    pub use crate::unit::*;
    pub use crate::view::View;
    pub use crate::views::*;

    pub use glam::*;
    pub use tracing::{debug, error, info, trace, warn};

    pub use ori_macro::{view, Build};
}
