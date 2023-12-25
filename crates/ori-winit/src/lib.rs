#![allow(clippy::module_inception)]
#![warn(missing_docs)]

//! Winit backend for Ori.

#[cfg(target_os = "android")]
mod android;
mod convert;
mod error;
mod launch;
mod launcher;
mod log;
mod util;
mod window;

#[cfg(feature = "wgpu")]
mod wgpu;

#[cfg(feature = "tracing")]
mod tracing;

pub use error::*;
pub use launcher::*;

#[cfg(feature = "wgpu")]
pub use wgpu::WgpuContext;

#[doc(hidden)]
pub mod __private {
    #[cfg(target_os = "android")]
    pub use crate::android::*;
}
