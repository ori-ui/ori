//! Wayland platform implementation.

mod app;
mod error;
mod xkb;

pub use app::launch;
pub use error::WaylandError;
