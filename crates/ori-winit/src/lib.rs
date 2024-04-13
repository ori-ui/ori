#![allow(clippy::module_inception)]
#![warn(missing_docs)]

//! Winit backend for Ori.

#[cfg(target_os = "android")]
mod android;
mod clipboard;
mod convert;
mod error;
mod launch;
mod tracing;
mod window;

pub use error::*;
pub use launch::launch;
pub use window::*;

#[cfg(feature = "wgpu")]
pub use ori_wgpu::WgpuContext;

#[doc(hidden)]
pub mod __private {
    #[cfg(target_os = "android")]
    pub use crate::android::*;
}

#[cfg(all(feature = "wgpu", feature = "glow"))]
compile_error!("The `wgpu` and `glow` features are mutually exclusive.");
