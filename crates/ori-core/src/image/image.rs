use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::Deref,
    sync::{Arc, Weak},
};

use crate::prelude::Color;

use super::ImageData;

/// Include an image.
///
/// Path is relative to the `CARGO_MANIFEST_DIR` environment variable.
#[macro_export]
#[cfg(feature = "image")]
macro_rules! include_image {
    ($path:literal) => {{
        static IMAGE: ::std::sync::OnceLock<$crate::image::Image> = ::std::sync::OnceLock::new();

        ::std::sync::OnceLock::get_or_init(&IMAGE, || {
            let bytes = <[::std::primitive::u8]>::to_vec(::std::include_bytes!(
                // use concat! to get the full path relative to the CARGO_MANIFEST_DIR
                ::std::concat!(::std::env!("CARGO_MANIFEST_DIR"), "/", $path)
            ));

            match $crate::image::Image::try_load_data(bytes) {
                ::std::result::Result::Ok(image) => image,
                ::std::result::Result::Err(err) => {
                    ::std::panic!("Failed to load image:{}: {}", $path, err);
                }
            }
        })
        .clone()
    }};
}

/// A unique identifier for an [`Image`].
///
/// The identifier is computed by hashing the image data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ImageId {
    pub(crate) hash: u64,
}

/// An clonable image.
#[derive(Clone, Debug)]
pub struct Image {
    id: ImageId,
    data: Arc<ImageData>,
}

impl Default for Image {
    fn default() -> Self {
        Self::from(ImageData::default())
    }
}

impl Image {
    /// Create a new image.
    ///
    /// # Panics
    /// - If `pixels.len()` is not equal to `width * height * 4`.
    pub fn new(pixels: Vec<u8>, width: u32, height: u32) -> Self {
        Self::from(ImageData::new(pixels, width, height))
    }

    /// Try to load an image from a file.
    #[cfg(feature = "image")]
    pub fn try_load_data(data: Vec<u8>) -> image::ImageResult<Self> {
        Ok(Self::from(ImageData::try_load_data(data)?))
    }

    /// Load an image from a file.
    #[cfg(feature = "image")]
    pub fn load_data(data: Vec<u8>) -> Self {
        Self::from(ImageData::load_data(data))
    }

    /// Try to load an image from a file.
    #[cfg(feature = "image")]
    pub fn try_load(path: impl AsRef<std::path::Path>) -> image::ImageResult<Self> {
        Ok(Self::from(ImageData::try_load(path)?))
    }

    /// Load an image from a file.
    #[cfg(feature = "image")]
    pub fn load(path: impl AsRef<std::path::Path>) -> Self {
        Self::from(ImageData::load(path))
    }

    /// Get the [`ImageId`].
    pub fn id(&self) -> ImageId {
        self.id
    }

    /// Modify the image data.
    pub fn modify(&mut self, f: impl FnOnce(&mut ImageData)) {
        f(Arc::make_mut(&mut self.data));
        self.id = self.data.compute_id();
    }

    /// Multiply the image with a color.
    pub fn multiply(&mut self, color: Color) {
        let [r, g, b, a] = color.to_rgba8();

        self.modify(|data| {
            for pixel in data.chunks_exact_mut(4) {
                pixel[0] = (pixel[0] as u16 * r as u16 / 255) as u8;
                pixel[1] = (pixel[1] as u16 * g as u16 / 255) as u8;
                pixel[2] = (pixel[2] as u16 * b as u16 / 255) as u8;
                pixel[3] = (pixel[3] as u16 * a as u16 / 255) as u8;
            }
        });
    }

    /// Downgrade the image to a weak reference.
    pub fn downgrade(&self) -> WeakImage {
        WeakImage {
            id: self.id,
            data: Arc::downgrade(&self.data),
        }
    }
}

impl Deref for Image {
    type Target = ImageData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl From<ImageData> for Image {
    fn from(value: ImageData) -> Self {
        let id = value.compute_id();
        let data = Arc::new(value);

        Self { id, data }
    }
}

impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Image {}

impl Hash for Image {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// A weak reference to an [`Image`].
#[derive(Clone, Debug)]
pub struct WeakImage {
    id: ImageId,
    data: Weak<ImageData>,
}

impl WeakImage {
    /// Get the [`ImageId`].
    pub fn id(&self) -> ImageId {
        self.id
    }

    /// Get the number of strong references to the image.
    pub fn strong_count(&self) -> usize {
        self.data.strong_count()
    }

    /// Get the number of weak references to the image.
    pub fn weak_count(&self) -> usize {
        self.data.weak_count()
    }
}

impl PartialEq for WeakImage {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for WeakImage {}

impl Hash for WeakImage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
