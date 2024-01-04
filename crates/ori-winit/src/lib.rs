#![allow(clippy::module_inception)]
#![warn(missing_docs)]

//! Winit backend for Ori.

#[cfg(target_os = "android")]
mod android;
mod convert;
mod error;
mod launch;
mod log;
mod shell;
mod window;

#[cfg(feature = "tracing")]
mod tracing;

pub use error::*;
pub use shell::*;

#[cfg(feature = "wgpu")]
pub use ori_wgpu::WgpuContext;

#[doc(hidden)]
pub mod __private {
    #[cfg(target_os = "android")]
    pub use crate::android::*;
}

#[cfg(all(feature = "wgpu", feature = "glow"))]
compile_error!("The `wgpu` and `glow` features are mutually exclusive.");
