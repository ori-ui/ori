use std::{collections::HashMap, fmt::Debug};

use etagere::{size2, AtlasAllocator};
use fontdue::{layout::GlyphRasterConfig, Font};
use glam::UVec2;

use crate::{
    image::{Image, ImageData},
    layout::{Rect, Size},
};

/// A font atlas managing a texture of rasterized glyphs.
#[derive(Clone)]
pub struct FontAtlas {
    allocator: AtlasAllocator,
    glyphs: HashMap<GlyphRasterConfig, Rect>,
    image: Image,
}

impl Debug for FontAtlas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontAtlas")
            .field("glyphs", &self.glyphs)
            .field("image", &self.image)
            .finish()
    }
}

impl Default for FontAtlas {
    fn default() -> Self {
        Self::new()
    }
}

impl FontAtlas {
    const PADDING: u32 = 2;

    /// Creates a new font atlas.
    pub fn new() -> Self {
        let allocator = AtlasAllocator::new(size2(1, 1));
        let data = HashMap::new();

        Self {
            allocator,
            glyphs: data,
            image: Image::default(),
        }
    }

    /// Returns the size of the atlas in pixels.
    pub fn size(&self) -> UVec2 {
        let size = self.allocator.size();
        UVec2::new(size.width as u32, size.height as u32)
    }

    /// Grows the atlas to the next power of two.
    pub fn grow(&mut self) {
        let size = if self.size().x == 0 {
            UVec2::splat(512)
        } else {
            self.size() * 2
        };

        // resize the allocator
        self.allocator = AtlasAllocator::new(size2(size.x as i32, size.y as i32));

        // resize the image
        let image_size = size.x as usize * size.y as usize * 4;
        let mut image_data = ImageData::new(vec![0; image_size], size.x, size.y);
        image_data.set_filter(false);
        self.image = Image::from(image_data);

        // clear the glyph cache
        self.glyphs.clear();
    }

    /// Rasterizes a glyph and returns its [`Rect`] in the atlas.
    ///
    /// Returns `None` if the atlas is full, in which case [`FontAtlas::grow`], should be called.
    pub fn glyph_rect(&mut self, font: &Font, config: GlyphRasterConfig) -> Option<Rect> {
        if let Some(&glyph) = self.glyphs.get(&config) {
            return Some(glyph);
        }

        let (metrics, pixels) = font.rasterize_config(config);

        if metrics.width == 0 || metrics.height == 0 {
            self.glyphs.insert(config, Rect::ZERO);
            return Some(Rect::ZERO);
        }

        let allocation_size = size2(
            metrics.width as i32 + Self::PADDING as i32 * 2,
            metrics.height as i32 + Self::PADDING as i32 * 2,
        );
        let allocation = self.allocator.allocate(allocation_size)?;

        let min = UVec2::new(
            allocation.rectangle.min.x as u32 + Self::PADDING,
            allocation.rectangle.min.y as u32 + Self::PADDING,
        );
        let size = Size::new(metrics.width as f32, metrics.height as f32);

        self.image.modify(|data| {
            for y in 0..metrics.height {
                for x in 0..metrics.width {
                    let index = y * metrics.width + x;
                    let pixel = pixels[index];

                    let x = x as u32 + min.x;
                    let y = y as u32 + min.y;
                    data.set_pixel(x, y, [255, 255, 255, pixel]);
                }
            }
        });

        let rect = Rect::min_size(min.as_vec2(), size);
        self.glyphs.insert(config, rect);

        Some(rect)
    }

    /// Rasterizes a glyph and returns its [`Rect`], in uv coodinates.
    ///
    /// Returns `None` if the atlas is full, in which case [`FontAtlas::grow`], should be called.
    pub fn glyph_rect_uv(&mut self, font: &Font, config: GlyphRasterConfig) -> Option<Rect> {
        let rect = self.glyph_rect(font, config)?;

        let size = self.size().as_vec2();
        let min = rect.min / size;
        let max = rect.max / size;

        Some(Rect::new(min, max))
    }

    /// Returns the image handle of the atlas.
    pub fn image(&self) -> &Image {
        &self.image
    }
}
