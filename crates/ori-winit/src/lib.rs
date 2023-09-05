#![allow(clippy::module_inception)]
#![warn(missing_docs)]

//! Winit backend for Ori.

#[cfg(target_os = "android")]
mod android;
mod app;
mod convert;
mod dummy;
mod error;
mod log;
mod proxy;
mod run;
mod window;

#[cfg(feature = "wgpu")]
mod wgpu;

#[cfg(feature = "tracing")]
mod tracing;

pub use app::*;
pub use error::*;

#[cfg(feature = "wgpu")]
type Render = crate::wgpu::WgpuRender;
#[cfg(not(feature = "wgpu"))]
type Render = crate::dummy::DummyRender;

#[doc(hidden)]

pub mod __private {
    #[cfg(target_os = "android")]
    pub use crate::android::*;
}
