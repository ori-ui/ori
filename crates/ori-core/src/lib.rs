mod build;
mod children;
mod context;
mod debug;
mod element;
mod event;
mod layout;
mod node;
mod style;
mod unit;
mod view;
mod views;
mod window;

pub use build::*;
pub use children::*;
pub use context::*;
pub use debug::*;
pub use element::*;
pub use event::*;
pub use layout::*;
pub use node::*;
pub use style::*;
pub use unit::*;
pub use view::*;
pub use views::*;
pub use window::*;

pub use glam as math;

pub mod prelude {
    pub use crate::build::Parent;
    pub use crate::children::{Children, FlexLayout};
    pub use crate::context::{Context, DrawContext, EventContext, LayoutContext};
    pub use crate::element::Element;
    pub use crate::event::{
        CloseWindow, Cursor, Key, KeyboardEvent, Modifiers, OpenWindow, PointerButton,
        PointerEvent, RequestRedrawEvent, WindowClosedEvent, WindowResizedEvent,
    };
    pub use crate::include_stylesheet;
    pub use crate::layout::{AlignItem, AvailableSpace, Axis, JustifyContent, Margin, Padding};
    pub use crate::node::Node;
    pub use crate::style::{trans, Style, Styleable, Stylesheet};
    pub use crate::unit::*;
    pub use crate::view::View;
    pub use crate::views::*;
    pub use crate::window::{ScopeWindowExt, Window, WindowId};

    pub use glam::*;
    pub use tracing::{debug, error, info, trace, warn};

    pub use ori_macro::{view, Build};
}
