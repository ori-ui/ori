use std::{collections::HashMap, fmt::Debug};

use etagere::{size2, AtlasAllocator};
use fontdue::{layout::GlyphRasterConfig, Font};
use glam::{UVec2, Vec2};

use crate::{ImageData, ImageHandle, Rect, Renderer};

/// A font atlas managing a texture of rasterized glyphs.
#[derive(Clone)]
pub struct FontAtlas {
    allocator: AtlasAllocator,
    glyphs: HashMap<GlyphRasterConfig, Rect>,
    image: Option<ImageHandle>,
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
            image: None,
        }
    }

    /// Returns the size of the atlas in pixels.
    pub fn size(&self) -> UVec2 {
        let size = self.allocator.size();
        UVec2::new(size.width as u32, size.height as u32)
    }

    /// Resizes the atlas.
    pub fn grow(&mut self, renderer: &dyn Renderer) {
        let size = if self.size().x == 0 {
            UVec2::splat(512)
        } else {
            self.size() * 2
        };

        // resize the allocator
        self.allocator = AtlasAllocator::new(size2(size.x as i32, size.y as i32));

        // resize the image
        let image_size = size.x as usize * size.y as usize * 4;
        let image_data = ImageData::new(size.x, size.y, vec![0; image_size]);
        self.image = Some(renderer.create_image(&image_data));

        // clear the glyph cache
        self.glyphs.clear();
    }

    /// Rasterizes a glyph and returns its [`Rect`] in the atlas.
    ///
    /// Returns `None` if the atlas is full, in which case [`FontAtlas::resize`], should be called.
    pub fn glyph_rect(
        &mut self,
        renderer: &dyn Renderer,
        font: &Font,
        config: GlyphRasterConfig,
    ) -> Option<Rect> {
        if let Some(&glyph) = self.glyphs.get(&config) {
            return Some(glyph);
        }

        let (metrics, pixels) = font.rasterize_config(config);

        if metrics.width == 0 || metrics.height == 0 {
            self.glyphs.insert(config, Rect::ZERO);
            return Some(Rect::ZERO);
        }

        let pixels: Vec<_> = (pixels.iter().flat_map(|&pixel| [255, 255, 255, pixel])).collect();
        let data = ImageData::new(metrics.width as u32, metrics.height as u32, pixels);

        let allocation_size = size2(
            metrics.width as i32 + Self::PADDING as i32 * 2,
            metrics.height as i32 + Self::PADDING as i32 * 2,
        );
        let allocation = self.allocator.allocate(allocation_size)?;

        let min = UVec2::new(
            allocation.rectangle.min.x as u32 + Self::PADDING,
            allocation.rectangle.min.y as u32 + Self::PADDING,
        );
        let size = Vec2::new(metrics.width as f32, metrics.height as f32);

        if let Some(image) = &mut self.image {
            renderer.write_image(image, min, &data);
        }

        let rect = Rect::min_size(min.as_vec2(), size);
        self.glyphs.insert(config, rect);

        Some(rect)
    }

    /// Rasterizes a glyph and returns its [`Rect`], in uv coodinates.
    ///
    /// Returns `None` if the atlas is full, in which case [`FontAtlas::resize`], should be called.
    pub fn glyph_rect_uv(
        &mut self,
        renderer: &dyn Renderer,
        font: &Font,
        config: GlyphRasterConfig,
    ) -> Option<Rect> {
        let rect = self.glyph_rect(renderer, font, config)?;

        let size = self.size().as_vec2();
        let min = rect.min / size;
        let max = rect.max / size;

        Some(Rect::new(min, max))
    }

    /// Returns the image handle of the atlas.
    pub fn image(&self) -> Option<&ImageHandle> {
        self.image.as_ref()
    }
}
