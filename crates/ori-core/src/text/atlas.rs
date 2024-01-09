use std::{collections::HashMap, fmt::Debug};

use cosmic_text::{CacheKey, FontSystem, LayoutGlyph, SwashCache, SwashContent};
use etagere::{size2, AtlasAllocator};

use crate::{
    image::{Image, ImageData},
    layout::{Point, Rect, Size, Vector},
};

/// A rasterized glyph.
#[derive(Clone, Copy, Debug)]
pub struct RasterizedGlyph {
    /// The UV coordinates of the glyph in the [`FontAtlas`].
    pub uv: Rect,
    /// The offset of the glyph.
    pub offset: Vector,
    /// The size of the glyph.
    pub size: Size,
}

impl RasterizedGlyph {
    /// A glyph that does nothing
    pub const NULL: Self = Self {
        uv: Rect::ZERO,
        offset: Vector::ZERO,
        size: Size::ZERO,
    };
}

/// A font atlas managing a texture of rasterized glyphs.
#[derive(Clone)]
pub struct FontAtlas {
    allocator: AtlasAllocator,
    glyphs: HashMap<CacheKey, RasterizedGlyph>,
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
    const PADDING: i32 = 2;

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
    pub fn size(&self) -> u32 {
        self.allocator.size().width as u32
    }

    /// Grows the atlas to the next power of two.
    pub fn grow(&mut self) {
        let size = if self.size() == 1 {
            512
        } else {
            self.size() * 2
        };

        // resize the allocator
        self.allocator = AtlasAllocator::new(size2(size as i32, size as i32));

        // resize the image
        let image_size = size as usize * size as usize * 4;
        let image_data = ImageData::new(vec![0; image_size], size, size);
        self.image = Image::from(image_data);

        // clear the glyph cache
        self.glyphs.clear();
    }

    /// Rasterizes a glyph and returns its [`Rect`] in the atlas.
    ///
    /// Returns `None` if the atlas is full, in which case [`FontAtlas::grow`], should be called.
    pub fn rasterize_glyph(
        &mut self,
        cache: &mut SwashCache,
        font_system: &mut FontSystem,
        glyph: &LayoutGlyph,
    ) -> Option<RasterizedGlyph> {
        let physical = glyph.physical((0.0, 0.0), 1.0);

        if let Some(&glyph) = self.glyphs.get(&physical.cache_key) {
            return Some(glyph);
        }

        let Some(image) = cache.get_image_uncached(font_system, physical.cache_key) else {
            panic!("failed to rasterize glyph");
        };

        if image.placement.width == 0 || image.placement.height == 0 {
            return Some(RasterizedGlyph::NULL);
        }

        let width = image.placement.width as i32;
        let height = image.placement.height as i32;
        let image_size = size2(width, height);
        let padding_size = size2(Self::PADDING, Self::PADDING) * 2;

        let alloc = self.allocator.allocate(image_size + padding_size)?;

        let min_x = alloc.rectangle.min.x;
        let min_y = alloc.rectangle.min.y;

        self.image.modify(|data| {
            for y in 0..height {
                for x in 0..width {
                    let i = (width * y + x) as usize;

                    let color = match image.content {
                        SwashContent::Mask => [255, 255, 255, image.data[i]],
                        SwashContent::SubpixelMask => todo!(),
                        SwashContent::Color => [
                            image.data[i * 4],
                            image.data[i * 4 + 1],
                            image.data[i * 4 + 2],
                            image.data[i * 4 + 3],
                        ],
                    };

                    let x = min_x + x + Self::PADDING;
                    let y = min_y + y + Self::PADDING;
                    data.set_pixel(x as u32, y as u32, color);
                }
            }
        });

        let size = Size::new(image.placement.width as f32, image.placement.height as f32);
        let offset = Vector::new(image.placement.left as f32, -image.placement.top as f32);

        let min = Point::new(min_x as f32, min_y as f32) + Self::PADDING as f32;
        let uv = Rect::min_size(min / self.size() as f32, size / self.size() as f32);
        let glyph = RasterizedGlyph { uv, offset, size };

        self.glyphs.insert(physical.cache_key, glyph);

        Some(glyph)
    }

    /// Returns the image handle of the atlas.
    pub fn image(&self) -> &Image {
        &self.image
    }
}
