use std::{
    any::Any,
    fmt::{Debug, Display},
    io,
    path::{Path, PathBuf},
    sync::{Arc, Weak},
};

use glam::Vec2;

#[macro_export]
macro_rules! include_image {
    ($($tt:tt)*) => {
        $crate::ImageData::from_bytes(include_bytes!($($tt)*))
    };
}

#[derive(Debug)]
pub enum ImageLoadError {
    Io(io::Error),
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ImageSource {
    #[cfg(feature = "image")]
    Path(PathBuf),
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
    pub fn try_load(self) -> Result<ImageData, ImageLoadError> {
        match self {
            Self::Data(image) => Ok(image),
            #[cfg(feature = "image")]
            Self::Path(path) => Ok(ImageData::try_load(path)?),
        }
    }

    pub fn load(self) -> ImageData {
        match self {
            Self::Data(image) => image,
            #[cfg(feature = "image")]
            Self::Path(path) => ImageData::load(path),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ImageData {
    width: u32,
    height: u32,
    pixels: Arc<[u8]>,
}

impl ImageData {
    pub fn new(width: u32, height: u32, pixels: impl Into<Arc<[u8]>>) -> Self {
        Self {
            width,
            height,
            pixels: pixels.into(),
        }
    }

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

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }

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

#[derive(Clone, Debug)]
pub struct ImageHandle {
    width: u32,
    height: u32,
    handle: Arc<dyn Any + Send + Sync>,
}

impl ImageHandle {
    pub fn new<T: Any + Send + Sync>(handle: T, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            handle: Arc::new(handle),
        }
    }

    pub fn downgrade(&self) -> WeakImageHandle {
        WeakImageHandle {
            width: self.width,
            height: self.height,
            handle: Arc::downgrade(&self.handle),
        }
    }

    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.handle.downcast_ref()
    }

    pub fn downcast_arc<T: Any + Send + Sync>(self) -> Option<Arc<T>> {
        Arc::downcast(self.handle).ok()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }
}

#[derive(Clone, Debug)]
pub struct WeakImageHandle {
    width: u32,
    height: u32,
    handle: Weak<dyn Any + Send + Sync>,
}

impl WeakImageHandle {
    pub fn upgrade(&self) -> Option<ImageHandle> {
        Some(ImageHandle {
            width: self.width,
            height: self.height,
            handle: self.handle.upgrade()?,
        })
    }

    pub fn is_alive(&self) -> bool {
        self.handle.strong_count() > 0
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }
}
