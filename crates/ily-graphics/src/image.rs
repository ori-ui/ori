use std::{any::Any, path::Path, sync::Arc};

use glam::Vec2;

#[macro_export]
macro_rules! include_image {
    ($($tt:tt)*) => {
        $crate::ImageData::from_bytes(include_bytes!($($tt)*))
    };
}

#[derive(Clone, Debug, PartialEq)]
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

    pub fn try_load(path: impl AsRef<Path>) -> image::ImageResult<Self> {
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

    pub fn load(path: impl AsRef<Path>) -> Self {
        match Self::try_load(path) {
            Ok(image) => image,
            Err(err) => {
                tracing::error!("Failed to load image: {}", err);
                Self::default()
            }
        }
    }

    pub fn try_from_bytes(bytes: &[u8]) -> image::ImageResult<Self> {
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

impl<T: AsRef<Path>> From<T> for ImageData {
    fn from(path: T) -> Self {
        Self::load(path)
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
}
