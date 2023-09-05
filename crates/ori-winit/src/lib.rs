#![allow(clippy::module_inception)]
#![warn(missing_docs)]

//! Winit backend for Ori.

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
#[cfg(target_os = "android")]
pub mod __private {
    pub use winit::platform::android::activity::AndroidApp;

    pub static ANDROID_APP: std::sync::OnceLock<AndroidApp> = std::sync::OnceLock::new();

    pub fn set_android_app(app: AndroidApp) {
        ANDROID_APP.set(app).unwrap();
    }

    pub fn get_android_app() -> AndroidApp {
        ANDROID_APP.get().unwrap().clone()
    }
}
