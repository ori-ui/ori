//! Android platform specific implementations.

mod error;
mod run;

pub use error::*;
pub use run::*;

#[doc(hidden)]
pub use android_activity::AndroidApp;

#[doc(hidden)]
pub static ANDROID_APP: std::sync::OnceLock<AndroidApp> = std::sync::OnceLock::new();
