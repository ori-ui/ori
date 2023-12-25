use std::fmt::Display;

use winit::error::{EventLoopError, OsError};

/// An error that can occur when running an application.
#[derive(Debug)]
pub enum Error {
    /// A glow render error.
    #[cfg(feature = "glow")]
    Glow(ori_glow::GlowError),
    /// A wgpu render error.
    #[cfg(feature = "wgpu")]
    Wgpu(ori_wgpu::WgpuError),
    /// An OS error.
    OsError(OsError),
    /// An error occurred with the event loop.
    EventLoop(EventLoopError),
}

#[cfg(feature = "glow")]
impl From<ori_glow::GlowError> for Error {
    fn from(err: ori_glow::GlowError) -> Self {
        Self::Glow(err)
    }
}

#[cfg(feature = "wgpu")]
impl From<ori_wgpu::WgpuError> for Error {
    fn from(err: ori_wgpu::WgpuError) -> Self {
        Self::Wgpu(err)
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
            Error::Glow(err) => write!(f, "{}", err),
            #[cfg(feature = "wgpu")]
            Error::Wgpu(err) => write!(f, "{}", err),
            Error::OsError(err) => write!(f, "{}", err),
            Error::EventLoop(err) => write!(f, "{}", err),
        }
    }
}
