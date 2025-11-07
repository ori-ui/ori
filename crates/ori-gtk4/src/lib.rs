#![allow(clippy::type_complexity)]

mod app;
mod context;
mod view;
mod window;

pub use app::*;
pub use context::*;
pub use view::*;
pub use window::*;

pub use ori as core;
pub use ori::Action;

pub mod prelude {
    pub use crate::{
        app::{App, Error},
        context::Context,
        view::*,
        window::Window,
    };
}
