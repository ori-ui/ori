use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

use crate::{ImageId, Size};

/// Image data.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ImageData {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    filter: bool,
}

impl Default for ImageData {
    fn default() -> Self {
        Self::new(vec![255; 4], 1, 1)
    }
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

    /// Try to load image data from a file.
    #[cfg(feature = "image")]
    pub fn try_load_data(data: Vec<u8>) -> image::ImageResult<Self> {
        let data = image::load_from_memory(&data)?;

        Ok(Self {
            pixels: data.to_rgba8().into_raw(),
            width: data.width(),
            height: data.height(),
            filter: true,
        })
    }

    /// Load image data from a file.
    #[cfg(feature = "image")]
    pub fn load_data(data: Vec<u8>) -> Self {
        match Self::try_load_data(data) {
            Ok(data) => data,
            Err(err) => {
                eprintln!("Failed to load image data: {}", err);
                Self::default()
            }
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
