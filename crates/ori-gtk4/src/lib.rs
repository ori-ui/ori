#![allow(clippy::type_complexity)]

mod app;
mod context;
mod view;
mod window;

#[path = "views/mod.rs"]
mod gtk4_views;

pub mod views {
    pub use crate::gtk4_views::*;
    pub use ori::views::*;
}

pub use app::*;
pub use context::*;
pub use view::*;
pub use window::*;

pub use ori as core;

pub mod prelude {
    pub use crate::{App, Context, Error, View, ViewSeq, Window, views::*};
}
