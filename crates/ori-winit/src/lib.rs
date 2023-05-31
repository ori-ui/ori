//! A [`winit`] backend for Ori.
//!
//! See [`WinitBackend`] and [`App`] for more information.

mod app;
mod backend;
mod convert;

pub use app::*;
pub use backend::*;

pub mod prelude {
    //! A collection of commonly used types and traits.

    pub use crate::app::App;
}
