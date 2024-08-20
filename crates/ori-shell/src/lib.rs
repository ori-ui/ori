#![allow(clippy::module_inception)]
#![warn(missing_docs)]

//! X11 backend for Ori.

use ori_app::{AppBuilder, IntoUiBuilder};
use ori_core::window::Window;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

pub mod platform;

/// Errors that can occur when using ori-shell.
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// X11 error.
    #[cfg(x11_platform)]
    X11(platform::x11::X11Error),

    /// Wayland error.
    #[cfg(wayland_platform)]
    Wayland(platform::wayland::WaylandError),

    /// No platform feature enabled.
    NoPlatform,
}

#[cfg(x11_platform)]
impl From<platform::x11::X11Error> for Error {
    fn from(err: platform::x11::X11Error) -> Self {
        Self::X11(err)
    }
}

#[cfg(wayland_platform)]
impl From<platform::wayland::WaylandError> for Error {
    fn from(err: platform::wayland::WaylandError) -> Self {
        Self::Wayland(err)
    }
}

impl std::fmt::Display for Error {
    #[allow(unused_variables, unreachable_patterns)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(x11_platform)]
            Error::X11(err) => write!(f, "{}", err),

            #[cfg(wayland_platform)]
            Error::Wayland(err) => write!(f, "{}", err),

            Error::NoPlatform => write!(f, "no platform feature enabled"),

            _ => unreachable!(),
        }
    }
}

impl std::error::Error for Error {}

/// Launch an Ori application.
#[allow(unused_variables)]
pub fn launch<T>(app: AppBuilder<T>, data: T) -> Result<(), Error> {
    let mut filter = EnvFilter::default().add_directive(tracing::Level::DEBUG.into());

    if let Ok(env) = std::env::var("RUST_LOG") {
        filter = filter.add_directive(env.parse().unwrap());
    }

    let subscriber = tracing_subscriber::registry().with(filter);

    #[cfg(not(target_arch = "wasm32"))]
    let subscriber = {
        let fmt_layer = tracing_subscriber::fmt::Layer::default();
        subscriber.with(fmt_layer)
    };

    #[cfg(target_arch = "wasm32")]
    let subscriber = {
        let wasm_layer = tracing_wasm::WASMLayer::new(Default::default());
        subscriber.with(fmt_layer)
    };

    if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("Failed to set global default subscriber: {}", err);
    }

    #[cfg(x11_platform)]
    {
        let app = platform::x11::X11App::new(app, data)?;
        return Ok(app.run()?);
    }

    #[cfg(wayland_platform)]
    {
        platform::wayland::launch(app, data)?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err(Error::NoPlatform)
}

/// Launch an Ori simple application.
pub fn launch_simple<V, P>(
    window: Window,
    ui: impl IntoUiBuilder<V, P, Data = ()>,
) -> Result<(), Error> {
    let app = AppBuilder::new().window(window, ui);
    launch(app, ())
}
