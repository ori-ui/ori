use std::{
    any::Any,
    fmt::{Debug, Display},
    io,
    path::{Path, PathBuf},
    sync::{Arc, Weak},
};

use glam::Vec2;

/// Includes [`ImageData`] from a file.
#[macro_export]
macro_rules! include_image {
    ($($tt:tt)*) => {
        $crate::ImageData::from_bytes(include_bytes!($($tt)*))
    };
}

/// An error that can occur when loading an image.
#[derive(Debug)]
pub enum ImageLoadError {
    /// An IO error occurred.
    Io(io::Error),
    /// An [`image::ImageError`] error occurred.
    #[cfg(feature = "image")]
    Image(image::ImageError),
}

impl From<io::Error> for ImageLoadError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

#[cfg(feature = "image")]
impl From<image::ImageError> for ImageLoadError {
    fn from(err: image::ImageError) -> Self {
        Self::Image(err)
    }
}

impl Display for ImageLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {}", err),
            #[cfg(feature = "image")]
            Self::Image(err) => write!(f, "Image error: {}", err),
        }
    }
}

/// A source for an image.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ImageSource {
    /// A path to an image.
    ///
    /// Requires the `image` feature.
    #[cfg(feature = "image")]
    Path(PathBuf),
    /// Image data.
    Data(ImageData),
}

impl Default for ImageSource {
    fn default() -> Self {
        Self::Data(ImageData::default())
    }
}

#[cfg(feature = "image")]
impl From<PathBuf> for ImageSource {
    fn from(path: PathBuf) -> Self {
        Self::Path(path)
    }
}

#[cfg(feature = "image")]
impl From<&Path> for ImageSource {
    fn from(path: &Path) -> Self {
        Self::Path(path.into())
    }
}

#[cfg(feature = "image")]
impl From<String> for ImageSource {
    fn from(path: String) -> Self {
        Self::Path(path.into())
    }
}

#[cfg(feature = "image")]
impl From<&str> for ImageSource {
    fn from(path: &str) -> Self {
        Self::Path(path.into())
    }
}

impl From<ImageData> for ImageSource {
    fn from(image: ImageData) -> Self {
        Self::Data(image)
    }
}

#[cfg(feature = "image")]
impl From<&[u8]> for ImageSource {
    fn from(bytes: &[u8]) -> Self {
        Self::Data(ImageData::from_bytes(bytes))
    }
}

impl ImageSource {
    /// Tries to load the [`ImageData`] from the source.
    pub fn try_load(self) -> Result<ImageData, ImageLoadError> {
        match self {
            Self::Data(image) => Ok(image),
            #[cfg(feature = "image")]
            Self::Path(path) => Ok(ImageData::try_load(path)?),
        }
    }

    /// Loads the [`ImageData`] from the source.
    ///
    /// # Panics
    /// - If loading the image fails.
    pub fn load(self) -> ImageData {
        match self {
            Self::Data(image) => image,
            #[cfg(feature = "image")]
            Self::Path(path) => ImageData::load(path),
        }
    }
}

/// Image data, see [`ImageSource`] and [`ImageHandle`].
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ImageData {
    width: u32,
    height: u32,
    pixels: Arc<[u8]>,
}

impl ImageData {
    /// Creates a new image.
    ///
    /// # Panics
    /// - If the length of `pixels` is not equal to `width * height * 4`.
    pub fn new(width: u32, height: u32, pixels: impl Into<Arc<[u8]>>) -> Self {
        let pixels = pixels.into();

        assert!(
            pixels.len() == (width * height * 4) as usize,
            "The length of `pixels` must be equal to `width * height * 4`"
        );

        Self {
            width,
            height,
            pixels,
        }
    }

    /// Tries to load an image from a path.
    ///
    /// Requires the `image` feature.
    #[cfg(feature = "image")]
    pub fn try_load(path: impl AsRef<Path>) -> Result<Self, ImageLoadError> {
        let image = image::open(path)?;
        let width = image.width();
        let height = image.height();
        let pixels = image.into_rgba8().into_raw();

        Ok(Self {
            width,
            height,
            pixels: pixels.into(),
        })
    }

    /// Loads an image from a path.
    ///
    /// # Panics
    /// - If loading the image fails.
    ///
    /// Requires the `image` feature.
    #[cfg(feature = "image")]
    pub fn load(path: impl AsRef<Path>) -> Self {
        match Self::try_load(path) {
            Ok(image) => image,
            Err(err) => {
                tracing::error!("Failed to load image: {}", err);
                Self::default()
            }
        }
    }

    /// Tries to load an image from bytes.
    ///
    /// Requires the `image` feature.
    #[cfg(feature = "image")]
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, ImageLoadError> {
        let image = image::load_from_memory(bytes)?;
        let width = image.width();
        let height = image.height();
        let pixels = image.into_rgba8().into_raw();

        Ok(Self {
            width,
            height,
            pixels: pixels.into(),
        })
    }

    /// Loads an image from bytes.
    ///
    /// # Panics
    /// - If loading the image fails.
    ///
    /// Requires the `image` feature.
    #[cfg(feature = "image")]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        match Self::try_from_bytes(bytes) {
            Ok(image) => image,
            Err(err) => {
                tracing::error!("Failed to load image: {}", err);
                Self::default()
            }
        }
    }

    /// Returns the width of the image.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the image.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns the size of the image.
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }

    /// Returns the pixels of the image.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

impl Default for ImageData {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            pixels: vec![0, 0, 0, 0].into(),
        }
    }
}

impl Debug for ImageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageData")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("pixels", &Arc::as_ptr(&self.pixels))
            .finish()
    }
}

/// A handle to a loaded image.
#[derive(Clone, Debug)]
pub struct ImageHandle {
    width: u32,
    height: u32,
    handle: Arc<dyn Any + Send + Sync>,
}

impl ImageHandle {
    /// Creates a new image handle. This is called by [`Renderer::create_image`](crate::Renderer::create_image)
    /// and should usually not be called manually.
    pub fn new<T: Any + Send + Sync>(handle: T, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            handle: Arc::new(handle),
        }
    }

    /// Downgrades the image handle to a [`WeakImageHandle`].
    pub fn downgrade(&self) -> WeakImageHandle {
        WeakImageHandle {
            width: self.width,
            height: self.height,
            handle: Arc::downgrade(&self.handle),
        }
    }

    /// Tries to downcast the image handle to a concrete type.
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.handle.downcast_ref()
    }

    /// Tries to downcast the image handle to a concrete type.
    pub fn downcast_arc<T: Any + Send + Sync>(self) -> Option<Arc<T>> {
        Arc::downcast(self.handle).ok()
    }

    /// Returns the width of the image.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the image.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns the size of the image.
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }
}

/// A weak handle to a loaded image, see [`ImageHandle::downgrade`].
#[derive(Clone, Debug)]
pub struct WeakImageHandle {
    width: u32,
    height: u32,
    handle: Weak<dyn Any + Send + Sync>,
}

impl WeakImageHandle {
    /// Upgrades the image handle to an [`ImageHandle`].
    pub fn upgrade(&self) -> Option<ImageHandle> {
        Some(ImageHandle {
            width: self.width,
            height: self.height,
            handle: self.handle.upgrade()?,
        })
    }

    /// Returns true if the image is still alive, and can be upgraded.
    pub fn is_alive(&self) -> bool {
        self.handle.strong_count() > 0
    }

    /// Returns the width of the image.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the image.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns the size of the image.
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }
}
