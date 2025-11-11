#![allow(clippy::type_complexity)]

mod app;
mod context;
mod view;

#[path = "views/mod.rs"]
mod gtk4_views;

pub mod views {
    pub use crate::gtk4_views::*;
    pub use ori::views::*;
}

pub use app::*;
pub use context::*;
pub use view::*;

pub use ori::{self as core, Action, Event};

pub mod prelude {
    pub use crate::{Action, App, Context, Effect, Error, View, include_css, views::*};

    pub use ori::{Event, Key, Proxy};
}
