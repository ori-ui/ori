use std::collections::HashMap;

use cosmic_text::{CacheKey, FontSystem, SwashCache, SwashContent};

use crate::{
    image::Image,
    layout::{Point, Rect, Size},
};

/// A font atlas.
#[derive(Debug)]
pub struct FontAtlas {
    image: Image,
    root: Node,
    glyphs: HashMap<CacheKey, AtlasGlyph>,
}

impl FontAtlas {
    /// Create a new font atlas with the given size.
    pub fn new(size: u32) -> Self {
        FontAtlas {
            image: Image::new(vec![0; size as usize * size as usize * 4], size, size),
            root: Node::new(0, 0, size, size),
            glyphs: HashMap::new(),
        }
    }

    /// Get the image of the atlas.
    pub fn image(&self) -> &Image {
        &self.image
    }

    fn grow(&mut self) {
        let new_size = self.image.width() * 2;
        let mut new_atlas = FontAtlas::new(new_size);

        for (key, glyph) in self.glyphs.drain() {
            let [rx, ry, _, _] = new_atlas.root.insert(glyph.width, glyph.height).unwrap();

            new_atlas.image.modify(|data| {
                Self::copy_sub_image(
                    self.image.data(),
                    data,
                    glyph.x as usize,
                    glyph.y as usize,
                    glyph.width as usize,
                    glyph.height as usize,
                    rx as usize,
                    ry as usize,
                    new_size as usize,
                );
            });

            let glyph = AtlasGlyph {
                uv: Rect::min_size(
                    Point::new(rx as f32, ry as f32),
                    Size::new(glyph.width as f32, glyph.height as f32),
                ),
                layout: glyph.layout,
                x: rx,
                y: ry,
                width: glyph.width,
                height: glyph.height,
            };

            new_atlas.glyphs.insert(key, glyph);
        }

        *self = new_atlas;
    }

    #[allow(clippy::too_many_arguments)]
    fn copy_sub_image(
        src: &[u8],
        dst: &mut [u8],
        src_x: usize,
        src_y: usize,
        src_w: usize,
        src_h: usize,
        dst_x: usize,
        dst_y: usize,
        dst_w: usize,
    ) {
        for y in 0..src_h {
            for x in 0..src_w {
                let src_i = ((src_y + y) * src_w + src_x + x) * 4;
                let dst_i = ((dst_y + y) * dst_w + dst_x + x) * 4;

                dst[dst_i..dst_i + 4].copy_from_slice(&src[src_i..src_i + 4]);
            }
        }
    }

    /// Insert a physical glyph into the atlas.
    pub fn insert(
        &mut self,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        cache_key: CacheKey,
    ) -> Option<AtlasGlyph> {
        if let Some(glyph) = self.glyphs.get(&cache_key).copied() {
            return Some(glyph);
        }

        let image = swash_cache.get_image(font_system, cache_key).as_ref()?;

        let width = image.placement.width + 4;
        let height = image.placement.height + 4;

        // grow the atlas if necessary
        let [rx, ry, _, _] = loop {
            match self.root.insert(width, height) {
                Some(rect) => break rect,
                None => self.grow(),
            }
        };

        let image_width = self.image.width();

        self.image.modify(|data| {
            for y in 0..image.placement.height {
                for x in 0..image.placement.width {
                    let i = ((ry + y + 2) * image_width + rx + x + 2) as usize * 4;
                    let j = (y * image.placement.width + x) as usize * 4;

                    match image.content {
                        SwashContent::Mask => {
                            data[i] = 255;
                            data[i + 1] = 255;
                            data[i + 2] = 255;
                            data[i + 3] = image.data[j / 4];
                        }
                        SwashContent::SubpixelMask | SwashContent::Color => {
                            data[i..i + 4].copy_from_slice(&image.data[j..j + 4]);
                        }
                    }
                }
            }
        });

        let glyph = AtlasGlyph {
            uv: Rect::min_size(
                Point::new((rx + 2) as f32, (ry + 2) as f32),
                Size::new(width as f32, height as f32),
            ),
            layout: Rect::min_size(
                Point::new(image.placement.left as f32, image.placement.top as f32),
                Size::new(image.placement.width as f32, image.placement.height as f32),
            ),
            x: rx,
            y: ry,
            width,
            height,
        };

        Some(*self.glyphs.entry(cache_key).or_insert(glyph))
    }
}

/// A glyph in the font atlas.
#[derive(Clone, Copy, Debug)]
pub struct AtlasGlyph {
    /// The uv coordinates of the glyph in the atlas.
    pub uv: Rect,

    /// The layout rect of the glyph.
    pub layout: Rect,

    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Debug)]
struct Node {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    children: Option<(Box<Node>, Box<Node>)>,
}

impl Node {
    fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Node {
            x,
            y,
            width,
            height,
            children: None,
        }
    }

    fn insert(&mut self, width: u32, height: u32) -> Option<[u32; 4]> {
        if let Some((ref mut right, ref mut down)) = self.children {
            if let Some(rect) = right.insert(width, height) {
                return Some(rect);
            }

            if let Some(rect) = down.insert(width, height) {
                return Some(rect);
            }

            return None;
        }

        if self.width < width || self.height < height {
            return None;
        }

        let rect = [self.x, self.y, width, height];

        let right = Box::new(Node::new(
            self.x + width,
            self.y,
            self.width - width,
            height,
        ));
        let down = Box::new(Node::new(
            self.x,
            self.y + height,
            self.width,
            self.height - height,
        ));

        self.width = width;
        self.height = height;
        self.children = Some((right, down));

        Some(rect)
    }
}
