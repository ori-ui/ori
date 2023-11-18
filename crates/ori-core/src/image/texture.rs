use super::Image;

/// An opaque backend texture identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureId {
    index: u64,
}

impl TextureId {
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

impl From<Image> for Texture {
    fn from(image: Image) -> Self {
        Self::Image(image)
    }
}
