use std::{collections::HashMap, hash::BuildHasherDefault, io, sync::Arc};

use cosmic_text::{Buffer, CacheKey, Command, FontSystem, SwashCache};
use ori_macro::include_font;
use tracing::debug;

use crate::{
    canvas::{AntiAlias, Canvas, Curve, FillRule, Paint},
    layout::{Affine, Point, Size, Vector},
};

use super::FontSource;

/// A context for loading and rasterizing fonts.
///
/// This is a wrapper around the [`cosmic_text`] crate, and provides a simple interface for
/// loading and rasterizing fonts. Interacting with this directly is not necessary for most
/// applications, see [`Text`](crate::views::Text) and [`TextInput`](crate::views::TextInput).
#[derive(Debug)]
pub struct Fonts {
    /// The swash cache.
    pub swash_cache: SwashCache,

    /// The font system.
    pub font_system: FontSystem,

    /// The glyph cache.
    pub glyph_cache: HashMap<CacheKey, Arc<Curve>, BuildHasherDefault<seahash::SeaHasher>>,
}

impl Default for Fonts {
    fn default() -> Self {
        Self::new()
    }
}

impl Fonts {
    /// Creates a new font context.
    pub fn new() -> Self {
        let mut fonts = Self {
            swash_cache: SwashCache::new(),
            font_system: FontSystem::new(),
            glyph_cache: HashMap::default(),
        };

        for font in fonts.font_system.db().faces() {
            for (family, _) in &font.families {
                debug!("Loaded font family: {}", family);
            }
        }

        fonts.load_font(include_font!("font")).unwrap();

        let db = fonts.font_system.db_mut();
        db.set_serif_family("Roboto");
        db.set_sans_serif_family("Roboto");
        db.set_monospace_family("Roboto Mono");
        db.set_cursive_family("Roboto");
        db.set_fantasy_family("Roboto");

        fonts
    }

    /// Loads a font from a [`FontSource`].
    ///
    /// This will usually either be a path to a font file or the font data itself, but can also
    /// be a [`Vec<FontSource>`] to load multiple fonts at once.
    pub fn load_font<'a>(&mut self, source: impl Into<FontSource<'a>>) -> Result<(), io::Error> {
        match source.into() {
            FontSource::Data(data) => {
                self.font_system.db_mut().load_font_data(data.to_vec());
            }
            FontSource::Path(path) => {
                self.font_system.db_mut().load_font_file(path)?;
            }
            FontSource::Bundle(data) => {
                let fonts = decompress_font_bundle(data.as_ref());

                for font in fonts {
                    let ids = self.font_system.db_mut().load_font_source(font);

                    for id in ids {
                        let face = self.font_system.db_mut().face(id).unwrap();

                        for (family, _) in &face.families {
                            debug!("Loaded font family: {}", family);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Calculates the size of a text buffer.
    ///
    /// The resulting size is the smallest rectangle that can contain the text,
    /// and is roughly equal to the widest line and the line height multiplied
    /// the number of laid out lines.
    pub fn buffer_size(buffer: &Buffer) -> Size {
        let mut width = 0.0;
        let mut height = 0.0;

        for run in buffer.layout_runs() {
            width = f32::max(width, run.line_w);
            height += buffer.metrics().line_height;
        }

        Size::new(width, height).ceil()
    }

    fn get_glyphs(&mut self, cache_key: CacheKey) -> Arc<Curve> {
        if let Some(curve) = self.glyph_cache.get(&cache_key).cloned() {
            return curve;
        }

        let commands = self
            .swash_cache
            .get_outline_commands(&mut self.font_system, cache_key);

        let mut curve = Curve::new();

        for command in commands.into_iter().flatten() {
            match command {
                Command::MoveTo(v) => {
                    let p = Point::new(v.x, -v.y);

                    curve.move_to(p);
                }
                Command::LineTo(v) => {
                    let p = Point::new(v.x, -v.y);

                    curve.line_to(p);
                }
                Command::QuadTo(v1, v2) => {
                    let p1 = Point::new(v1.x, -v1.y);
                    let p2 = Point::new(v2.x, -v2.y);

                    curve.quad_to(p1, p2);
                }
                Command::CurveTo(v1, v2, v3) => {
                    let p1 = Point::new(v1.x, -v1.y);
                    let p2 = Point::new(v2.x, -v2.y);
                    let p3 = Point::new(v3.x, -v3.y);

                    curve.cubic_to(p1, p2, p3);
                }
                Command::Close => {
                    curve.close();
                }
            }
        }

        self.glyph_cache
            .entry(cache_key)
            .or_insert(Arc::new(curve))
            .clone()
    }

    /// Rasterize a buffer.
    pub fn draw_buffer(
        &mut self,
        canvas: &mut Canvas,
        buffer: &Buffer,
        paint: Paint,
        offset: Vector,
        scale: f32,
    ) {
        let mut paint = paint;
        paint.anti_alias = AntiAlias::Full;

        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let physical = glyph.physical((0.0, 0.0), scale);
                let curve = self.get_glyphs(physical.cache_key);
                let offset = Vector::new(
                    glyph.x + glyph.x_offset,
                    glyph.y + run.line_y + glyph.y_offset,
                ) + offset;

                canvas.transform(Affine::translate(offset), |canvas| {
                    canvas.fill(curve.clone(), FillRule::NonZero, paint.clone());
                });
            }
        }
    }
}

fn decompress_font_bundle(bytes: &[u8]) -> Vec<cosmic_text::fontdb::Source> {
    let mut fonts = Vec::new();

    let data = miniz_oxide::inflate::decompress_to_vec(bytes).unwrap();
    let mut i = data.as_slice();

    let num_fonts = u32::from_le_bytes([i[0], i[1], i[2], i[3]]) as usize;
    i = &i[4..];

    for _ in 0..num_fonts {
        let len = u32::from_le_bytes([i[0], i[1], i[2], i[3]]) as usize;
        i = &i[4..];

        let data = Box::<[u8]>::from(&i[..len]);
        i = &i[len..];

        fonts.push(cosmic_text::fontdb::Source::Binary(Arc::new(data)));
    }

    fonts
}
