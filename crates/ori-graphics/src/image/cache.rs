use std::collections::HashMap;

use crate::{ImageHandle, ImageSource, WeakImageHandle};

/// A cache for images.
///
/// This is used to avoid loading the same image multiple times.
#[derive(Clone, Debug, Default)]
pub struct ImageCache {
    images: HashMap<ImageSource, WeakImageHandle>,
}

impl ImageCache {
    /// Creates a new image cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of images in the cache.
    pub fn len(&self) -> usize {
        self.images.len()
    }

    /// Returns `true` if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    /// Gets an image from the cache.
    pub fn get(&self, source: &ImageSource) -> Option<ImageHandle> {
        self.images.get(source)?.upgrade()
    }

    /// Inserts an image into the cache.
    pub fn insert(&mut self, source: ImageSource, handle: ImageHandle) {
        self.images.insert(source, handle.downgrade());
    }

    /// Clears the cache.
    pub fn clear(&mut self) {
        self.images.clear();
    }

    /// Removes all images that are no longer in use.
    pub fn clean(&mut self) {
        self.images.retain(|_, v| v.is_alive());
    }
}
