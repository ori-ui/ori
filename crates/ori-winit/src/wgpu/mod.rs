mod instance;
mod mesh;
mod quad;
mod render;
mod texture;

pub use instance::*;

pub use render::*;

use mesh::*;
use quad::*;
use texture::*;

use std::{collections::HashMap, sync::Arc};

use ori_core::image::TextureId;
use wgpu::{Device, Queue, TextureView};

unsafe fn bytes_of<T>(data: &T) -> &[u8] {
    std::slice::from_raw_parts(data as *const _ as *const u8, std::mem::size_of::<T>())
}

unsafe fn bytes_of_slice<T>(data: &[T]) -> &[u8] {
    std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data))
}

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
