use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    sync::{Arc, Weak},
};

use crate::Size;

/// A unique identifier for an [`Image`].
///
/// The identifier is computed by hashing the image data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ImageId {
    hash: u64,
}

/// Image data.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ImageData {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    filter: bool,
}

impl ImageData {
    /// Create a new image data.
    ///
    /// # Panics
    /// - If `pixels.len()` is not equal to `width * height * 4`.
    pub fn new(pixels: Vec<u8>, width: u32, height: u32) -> Self {
        assert_eq!(pixels.len() as u32, width * height * 4);

        Self {
            pixels,
            width,
            height,
            filter: true,
        }
    }

    /// Get the width of the image in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the height of the image in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the size of the image in pixels.
    pub fn size(&self) -> Size {
        Size::new(self.width as f32, self.height as f32)
    }

    /// Get a pixel.
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let i = (y * self.width + x) as usize * 4;
        let r = self.pixels[i];
        let g = self.pixels[i + 1];
        let b = self.pixels[i + 2];
        let a = self.pixels[i + 3];
        [r, g, b, a]
    }

    /// Set a pixel.
    pub fn set_pixel(&mut self, x: u32, y: u32, pixel: [u8; 4]) {
        let i = (y * self.width + x) as usize * 4;
        self.pixels[i] = pixel[0];
        self.pixels[i + 1] = pixel[1];
        self.pixels[i + 2] = pixel[2];
        self.pixels[i + 3] = pixel[3];
    }

    /// Get the pixels.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Get the pixels mutably.
    pub fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    /// Get the filter mode.
    ///
    /// If `true`, the image will be filtered with linear interpolation when scaled.
    /// If `false`, the image will be filtered with nearest neighbor interpolation when scaled.
    pub fn filter(&self) -> bool {
        self.filter
    }

    /// Set the filter mode.
    ///
    /// If `true`, the image will be filtered with linear interpolation when scaled.
    /// If `false`, the image will be filtered with nearest neighbor interpolation when scaled.
    pub fn set_filter(&mut self, filter: bool) {
        self.filter = filter;
    }

    /// Compute the id for this image data.
    ///
    /// **Note:** This is a relatively expensive operation.
    pub fn compute_id(&self) -> ImageId {
        let mut hasher = seahash::SeaHasher::new();
        self.hash(&mut hasher);
        ImageId {
            hash: hasher.finish(),
        }
    }
}

impl Debug for ImageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageData")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl Deref for ImageData {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.pixels
    }
}

impl DerefMut for ImageData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pixels
    }
}

/// An clonable image.
#[derive(Clone, Debug)]
pub struct Image {
    id: ImageId,
    data: Arc<ImageData>,
}

impl Default for Image {
    fn default() -> Self {
        Self::new(vec![255; 4], 1, 1)
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

    /// Get the [`ImageId`].
    pub fn id(&self) -> ImageId {
        self.id
    }

    /// Modify the image data.
    pub fn modify(&mut self, f: impl FnOnce(&mut ImageData)) {
        f(Arc::make_mut(&mut self.data));
        self.id = self.data.compute_id();
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
