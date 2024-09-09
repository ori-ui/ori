#![allow(clippy::module_inception)]
#![warn(missing_docs)]

//! X11 backend for Ori.

use ori_app::{AppBuilder, IntoUiBuilder};
use ori_core::window::Window;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

pub mod platform;

/// Errors that can occur when running an Ori application.
#[non_exhaustive]
#[derive(Debug)]
pub enum RunError {
    /// X11 error.
    #[cfg(x11_platform)]
    X11(platform::x11::X11Error),

    /// Wayland error.
    #[cfg(wayland_platform)]
    Wayland(platform::wayland::WaylandError),

    /// Android error.
    #[cfg(android_platform)]
    Android(platform::android::AndroidError),

    /// No platform feature enabled.
    NoPlatform,
}

#[cfg(x11_platform)]
impl From<platform::x11::X11Error> for RunError {
    fn from(err: platform::x11::X11Error) -> Self {
        Self::X11(err)
    }
}

#[cfg(wayland_platform)]
impl From<platform::wayland::WaylandError> for RunError {
    fn from(err: platform::wayland::WaylandError) -> Self {
        Self::Wayland(err)
    }
}

#[cfg(android_platform)]
impl From<platform::android::AndroidError> for RunError {
    fn from(err: platform::android::AndroidError) -> Self {
        Self::Android(err)
    }
}

impl std::fmt::Display for RunError {
    #[allow(unused_variables, unreachable_patterns)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(x11_platform)]
            RunError::X11(err) => write!(f, "{}", err),

            #[cfg(wayland_platform)]
            RunError::Wayland(err) => write!(f, "{}", err),

            #[cfg(android_platform)]
            RunError::Android(err) => write!(f, "{}", err),

            RunError::NoPlatform => write!(f, "no platform feature enabled"),

            _ => unreachable!(),
        }
    }
}

impl std::error::Error for RunError {}

/// Errors that can occur when installing the logger.
#[derive(Debug)]
pub enum LogError {
    /// Error parsing the log filter.
    FilterParseError(tracing_subscriber::filter::ParseError),

    /// Error setting the global default subscriber.
    SetGlobalError(tracing::subscriber::SetGlobalDefaultError),
}

impl From<tracing_subscriber::filter::ParseError> for LogError {
    fn from(err: tracing_subscriber::filter::ParseError) -> Self {
        Self::FilterParseError(err)
    }
}

impl From<tracing::subscriber::SetGlobalDefaultError> for LogError {
    fn from(err: tracing::subscriber::SetGlobalDefaultError) -> Self {
        Self::SetGlobalError(err)
    }
}

impl std::fmt::Display for LogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogError::FilterParseError(err) => write!(f, "{}", err),
            LogError::SetGlobalError(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for LogError {}

/// Install the default logger.
///
/// Logging in Ori is powered by the [`tracing`] crate. This function installs a logger with sane
/// defaults, however for anything more advanced you should use the [`tracing_subscriber`] crate
/// directly.
pub fn install_logger() -> Result<(), LogError> {
    let mut filter = EnvFilter::default().add_directive(tracing::Level::DEBUG.into());

    if let Ok(env) = std::env::var("RUST_LOG") {
        filter = filter.add_directive(env.parse()?);
    }

    let subscriber = tracing_subscriber::registry().with(filter);

    #[cfg(not(target_arch = "wasm32"))]
    let subscriber = subscriber.with(tracing_subscriber::fmt::Layer::default());

    #[cfg(target_arch = "wasm32")]
    let subscriber = subscriber.with(tracing_wasm::WASMLayer::new(Default::default()));

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

/// Run an Ori application.
#[allow(unused_variables, unreachable_code)]
pub fn run<T>(app: AppBuilder<T>, data: &mut T) -> Result<(), RunError> {
    #[cfg(wayland_platform)]
    if platform::wayland::is_available() {
        return Ok(platform::wayland::run(app, data)?);
    }

    #[cfg(x11_platform)]
    {
        return Ok(platform::x11::run(app, data)?);
    }

    #[cfg(android_platform)]
    {
        return Ok(platform::android::run(app, data)?);
    }

    #[allow(unreachable_code)]
    Err(RunError::NoPlatform)
}

/// Run an Ori simple application.
pub fn run_simple<V, P>(
    window: Window,
    ui: impl IntoUiBuilder<V, P, Data = ()>,
) -> Result<(), RunError> {
    let app = AppBuilder::new().window(window, ui);
    run(app, &mut ())
}
