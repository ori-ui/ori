use std::sync::atomic::{AtomicU64, Ordering};

use super::Image;

/// An opaque backend texture identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureId {
    index: u64,
}

impl Default for TextureId {
    fn default() -> Self {
        Self::from_index(0)
    }
}

impl TextureId {
    /// Create a new [`TextureId`].
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);

        let index = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self { index }
    }

    /// Create a new [`TextureId`] from an index.
    pub const fn from_index(index: u64) -> Self {
        Self { index }
    }

    /// Convert the [`TextureId`] to an index.
    pub const fn to_index(self) -> u64 {
        self.index
    }
}

/// A texture.
#[derive(Clone, Debug, PartialEq)]
pub enum Texture {
    /// An [`Image`] texture.
    Image(Image),
    /// A texture managed by the rendering backend.
    Backend(TextureId),
}

impl Default for Texture {
    fn default() -> Self {
        Self::Image(Image::default())
    }
}

impl From<Image> for Texture {
    fn from(image: Image) -> Self {
        Self::Image(image)
    }
}

impl From<TextureId> for Texture {
    fn from(id: TextureId) -> Self {
        Self::Backend(id)
    }
}
