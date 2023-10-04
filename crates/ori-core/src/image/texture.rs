#[cfg(feature = "wgpu")]
use std::sync::Arc;

use super::Image;

/// A texture.
#[cfg(feature = "wgpu")]
#[derive(Clone, Debug)]
pub struct WgpuTexture {
    texture: Arc<wgpu::Texture>,
}

#[cfg(feature = "wgpu")]
impl WgpuTexture {
    /// Create a new [`WgpuTexture`].
    pub fn new(texture: impl Into<Arc<wgpu::Texture>>) -> Self {
        Self {
            texture: texture.into(),
        }
    }
}

#[cfg(feature = "wgpu")]
impl std::ops::Deref for WgpuTexture {
    type Target = wgpu::Texture;

    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

#[cfg(feature = "wgpu")]
impl PartialEq for WgpuTexture {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.texture, &other.texture)
    }
}

#[cfg(feature = "wgpu")]
impl Eq for WgpuTexture {}

/// A texture.
#[derive(Clone, Debug, PartialEq)]
pub enum Texture {
    /// An image texture.
    Image(Image),
    /// A [`WgpuTexture`].
    #[cfg(feature = "wgpu")]
    Wgpu(WgpuTexture),
}

impl From<Image> for Texture {
    fn from(image: Image) -> Self {
        Self::Image(image)
    }
}

#[cfg(feature = "wgpu")]
impl From<WgpuTexture> for Texture {
    fn from(texture: WgpuTexture) -> Self {
        Self::Wgpu(texture)
    }
}
