use wayland_client::protocol::wl_surface;

use crate::platform::egl::EglError;

/// An error that can occur when working with Wayland.
#[derive(Debug)]
pub enum WaylandError {
    /// An error occurred when connecting to the Wayland server.
    Conntect(wayland_client::ConnectError),

    /// An error occurred with the Globals.
    Global(wayland_client::globals::GlobalError),

    /// An error occurred with the wl_surface protocol.
    Surface(wl_surface::Error),

    /// An error occurred with the wl_egl protocol.
    WlEgl(wayland_egl::Error),

    /// An error occurred with the egl.
    Egl(EglError),
}

impl From<wayland_client::ConnectError> for WaylandError {
    fn from(err: wayland_client::ConnectError) -> Self {
        Self::Conntect(err)
    }
}

impl From<wayland_client::globals::GlobalError> for WaylandError {
    fn from(err: wayland_client::globals::GlobalError) -> Self {
        Self::Global(err)
    }
}

impl From<wl_surface::Error> for WaylandError {
    fn from(err: wl_surface::Error) -> Self {
        Self::Surface(err)
    }
}

impl From<wayland_egl::Error> for WaylandError {
    fn from(err: wayland_egl::Error) -> Self {
        Self::WlEgl(err)
    }
}

impl From<EglError> for WaylandError {
    fn from(err: EglError) -> Self {
        Self::Egl(err)
    }
}

impl std::fmt::Display for WaylandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Conntect(err) => write!(f, "Wayland connection error: {}", err),
            Self::Global(err) => write!(f, "Wayland global error: {}", err),
            Self::Surface(err) => write!(f, "Wayland surface error: {:?}", err),
            Self::WlEgl(err) => write!(f, "Wayland EGL error: {}", err),
            Self::Egl(err) => write!(f, "EGL error: {}", err),
        }
    }
}

impl std::error::Error for WaylandError {}
