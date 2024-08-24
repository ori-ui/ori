//! X11 platform implementation.

mod clipboard;
mod error;
mod launch;
mod xkb;

pub use error::X11Error;
pub use launch::launch;
