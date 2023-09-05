#![allow(clippy::module_inception)]
#![warn(missing_docs)]

//! Winit backend for Ori.

#[cfg(target_os = "android")]
mod android;
mod app;
mod convert;
mod error;
mod proxy;
mod render;
mod run;
mod window;

#[cfg(feature = "tracing")]
mod tracing;

pub use app::*;
pub use error::*;

#[doc(hidden)]

pub mod __private {
    #[cfg(target_os = "android")]
    pub use crate::android::*;
}
