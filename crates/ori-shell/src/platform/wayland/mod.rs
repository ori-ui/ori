//! Wayland platform implementation.

mod error;
mod launch;
mod xkb;

pub use error::WaylandError;
pub use launch::launch;
