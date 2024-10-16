//! X11 platform implementation.

mod clipboard;
mod error;
mod run;

pub use error::X11Error;
pub use run::{run, X11RunOptions};
