use std::fmt::Display;

use winit::error::OsError;

/// An error that can occur when rendering.
#[derive(Debug)]
pub enum RenderError {
    /// Failed to create a surface.
    #[cfg(feature = "wgpu")]
    CreateSurface(wgpu::CreateSurfaceError),
    /// No adapter found.
    AdapterNotFound,
    /// Failed to request a device.
    #[cfg(feature = "wgpu")]
    RequestDevice(wgpu::RequestDeviceError),
    /// Surface incompatible with adapter.
    SurfaceIncompatible,
}

#[cfg(feature = "wgpu")]
impl From<wgpu::CreateSurfaceError> for RenderError {
    fn from(err: wgpu::CreateSurfaceError) -> Self {
        Self::CreateSurface(err)
    }
}

#[cfg(feature = "wgpu")]
impl From<wgpu::RequestDeviceError> for RenderError {
    fn from(err: wgpu::RequestDeviceError) -> Self {
        Self::RequestDevice(err)
    }
}

impl Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "wgpu")]
            RenderError::CreateSurface(err) => write!(f, "Failed to create surface: {}", err),
            RenderError::AdapterNotFound => write!(f, "No adapter found"),
            #[cfg(feature = "wgpu")]
            RenderError::RequestDevice(err) => write!(f, "Failed to request device: {}", err),
            RenderError::SurfaceIncompatible => write!(f, "Surface incompatible with adapter"),
        }
    }
}

/// An error that can occur when running an application.
#[derive(Debug)]
pub enum Error {
    /// A render error.
    Render(RenderError),
    /// An OS error.
    OsError(OsError),
}

impl From<RenderError> for Error {
    fn from(err: RenderError) -> Self {
        Self::Render(err)
    }
}

impl From<OsError> for Error {
    fn from(err: OsError) -> Self {
        Self::OsError(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Render(err) => write!(f, "{}", err),
            Error::OsError(err) => write!(f, "{}", err),
        }
    }
}
