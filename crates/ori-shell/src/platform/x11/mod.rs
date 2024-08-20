//! X11 platform implementation.

mod app;
mod clipboard;
mod error;
mod xkb;

pub use app::X11App;
pub use error::X11Error;
