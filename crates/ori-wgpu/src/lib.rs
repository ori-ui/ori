#![warn(missing_docs)]

//! A renderer using [`wgpu`].

mod instance;
mod mesh;
mod render;
mod texture;

pub use instance::*;
pub use render::*;

pub use wgpu::Surface;

use mesh::*;
use texture::*;

use std::{collections::HashMap, fmt::Display, sync::Arc};

use ori_core::image::TextureId;
use wgpu::{CreateSurfaceError, Device, Queue, RequestDeviceError, TextureView};

/// A context containing the [`wgpu::Device`] and [`wgpu::Queue`].
#[derive(Clone, Debug)]
pub struct WgpuContext {
    /// The [`wgpu::Device`] used for rendering.
    pub device: Arc<Device>,
    /// The [`wgpu::Queue`] used for rendering.
    pub queue: Arc<Queue>,
    /// The [`wgpu::TextureView`]s used for rendering.
    pub textures: HashMap<TextureId, Arc<TextureView>>,
}

/// An error that can occur when rendering.
#[derive(Debug)]
pub enum WgpuError {
    /// Failed to create a surface.
    CreateSurface(CreateSurfaceError),
    /// No adapter found.
    AdapterNotFound,
    /// Failed to request a device.
    RequestDevice(RequestDeviceError),
    /// Surface incompatible with adapter.
    SurfaceIncompatible,
}

impl From<CreateSurfaceError> for WgpuError {
    fn from(err: CreateSurfaceError) -> Self {
        Self::CreateSurface(err)
    }
}

impl From<RequestDeviceError> for WgpuError {
    fn from(err: RequestDeviceError) -> Self {
        Self::RequestDevice(err)
    }
}

impl Display for WgpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WgpuError::CreateSurface(err) => write!(f, "Failed to create surface: {}", err),
            WgpuError::AdapterNotFound => write!(f, "No adapter found"),
            WgpuError::RequestDevice(err) => write!(f, "Failed to request device: {}", err),
            WgpuError::SurfaceIncompatible => write!(f, "Surface incompatible with adapter"),
        }
    }
}
