#![allow(clippy::module_inception)]
#![warn(missing_docs)]

//! X11 backend for Ori.

use ori_app::{AppBuilder, IntoUiBuilder};
use ori_core::window::Window;

pub mod platform;

/// Errors that can occur when using ori-shell.
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// X11 error.
    #[cfg(x11_platform)]
    X11(platform::x11::X11Error),

    /// No platform feature enabled.
    NoPlatform,
}

#[cfg(x11_platform)]
impl From<platform::x11::X11Error> for Error {
    fn from(err: platform::x11::X11Error) -> Self {
        Self::X11(err)
    }
}

impl std::fmt::Display for Error {
    #[allow(unused_variables, unreachable_patterns)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(x11_platform)]
            Error::X11(err) => write!(f, "{}", err),
            Error::NoPlatform => write!(f, "no platform feature enabled"),
            _ => unreachable!(),
        }
    }
}

impl std::error::Error for Error {}

/// Launch an Ori application.
#[allow(unused_variables)]
pub fn launch<T>(app: AppBuilder<T>, data: T) -> Result<(), Error> {
    #[cfg(x11_platform)]
    {
        let app = platform::x11::X11App::new(app, data)?;
        return Ok(app.run()?);
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
