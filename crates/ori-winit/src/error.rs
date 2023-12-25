use std::fmt::Display;

use winit::error::{EventLoopError, OsError};

/// An error that can occur when running an application.
#[derive(Debug)]
pub enum Error {
    /// A glow render error.
    #[cfg(feature = "glow")]
    Render(crate::glow::GlowError),
    /// A wgpu render error.
    #[cfg(feature = "wgpu")]
    Render(crate::wgpu::WgpuError),
    /// An OS error.
    OsError(OsError),
    /// An error occurred with the event loop.
    EventLoop(EventLoopError),
}

#[cfg(feature = "glow")]
impl From<crate::glow::GlowError> for Error {
    fn from(err: crate::glow::GlowError) -> Self {
        Self::Render(err)
    }
}

#[cfg(feature = "wgpu")]
impl From<crate::wgpu::WgpuError> for Error {
    fn from(err: crate::wgpu::WgpuError) -> Self {
        Self::Render(err)
    }
}

impl From<OsError> for Error {
    fn from(err: OsError) -> Self {
        Self::OsError(err)
    }
}

impl From<EventLoopError> for Error {
    fn from(err: EventLoopError) -> Self {
        Self::EventLoop(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "glow")]
            Error::Render(err) => write!(f, "{}", err),
            #[cfg(feature = "wgpu")]
            Error::Render(err) => write!(f, "{}", err),
            Error::OsError(err) => write!(f, "{}", err),
            Error::EventLoop(err) => write!(f, "{}", err),
        }
    }
}
