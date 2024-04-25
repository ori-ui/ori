use std::{io, sync::Arc};

use cosmic_text::{fontdb::Source, Buffer, FontSystem, SwashCache};

const ROBOTO_BLACK: &[u8] = include_bytes!("../../font/Roboto-Black.ttf");
const ROBOTO_BLACK_ITALIC: &[u8] = include_bytes!("../../font/Roboto-BlackItalic.ttf");
const ROBOTO_BOLD: &[u8] = include_bytes!("../../font/Roboto-Bold.ttf");
const ROBOTO_BOLD_ITALIC: &[u8] = include_bytes!("../../font/Roboto-BoldItalic.ttf");
const ROBOTO_ITALIC: &[u8] = include_bytes!("../../font/Roboto-Italic.ttf");
const ROBOTO_LIGHT: &[u8] = include_bytes!("../../font/Roboto-Light.ttf");
const ROBOTO_LIGHT_ITALIC: &[u8] = include_bytes!("../../font/Roboto-LightItalic.ttf");
const ROBOTO_MEDIUM: &[u8] = include_bytes!("../../font/Roboto-Medium.ttf");
const ROBOTO_MEDIUM_ITALIC: &[u8] = include_bytes!("../../font/Roboto-MediumItalic.ttf");
const ROBOTO_REGULAR: &[u8] = include_bytes!("../../font/Roboto-Regular.ttf");
const ROBOTO_THIN: &[u8] = include_bytes!("../../font/Roboto-Thin.ttf");
const ROBOTO_THIN_ITALIC: &[u8] = include_bytes!("../../font/Roboto-ThinItalic.ttf");

const ROBOTO_MONO: &[u8] = include_bytes!("../../font/RobotoMono.ttf");
const ROBOTO_MONO_ITALIC: &[u8] = include_bytes!("../../font/RobotoMono-Italic.ttf");

const EMBEDDED_FONTS: &[&[u8]] = &[
    ROBOTO_BLACK,
    ROBOTO_BLACK_ITALIC,
    ROBOTO_BOLD,
    ROBOTO_BOLD_ITALIC,
    ROBOTO_ITALIC,
    ROBOTO_LIGHT,
    ROBOTO_LIGHT_ITALIC,
    ROBOTO_MEDIUM,
    ROBOTO_MEDIUM_ITALIC,
    ROBOTO_REGULAR,
    ROBOTO_THIN,
    ROBOTO_THIN_ITALIC,
    ROBOTO_MONO,
    ROBOTO_MONO_ITALIC,
];

use crate::{
    canvas::{Color, Mesh, Vertex},
    layout::{Point, Rect, Size},
};

use super::{FontAtlas, FontSource};

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
    /// The font atlas.
    pub font_atlas: FontAtlas,
}

impl Default for Fonts {
    fn default() -> Self {
        Self::new()
    }
}

impl Fonts {
    /// Creates a new font context.
    pub fn new() -> Self {
        let swash_cache = SwashCache::new();
        let font_atlas = FontAtlas::new();

        let mut fonts = Vec::new();

        for font in EMBEDDED_FONTS {
            fonts.push(Source::Binary(Arc::new(font.to_vec())));
        }

        let mut font_system = FontSystem::new_with_fonts(fonts);
        let db = font_system.db_mut();

        db.set_serif_family("Roboto");
        db.set_sans_serif_family("Roboto");
        db.set_monospace_family("Roboto Mono");
        db.set_cursive_family("Roboto");
        db.set_fantasy_family("Roboto");

        Self {
            swash_cache,
            font_system,
            font_atlas,
        }
    }

    /// Loads a font from a [`FontSource`].
    ///
    /// This will usually either be a path to a font file or the font data itself, but can also
    /// be a [`Vec<FontSource>`] to load multiple fonts at once.
    pub fn load_font(&mut self, source: impl Into<FontSource>) -> Result<(), io::Error> {
        match source.into() {
            FontSource::Data(data) => {
                self.font_system.db_mut().load_font_data(data);
            }
            FontSource::Path(path) => {
                self.font_system.db_mut().load_font_file(path)?;
            }
            FontSource::Set(sources) => {
                for source in sources {
                    self.load_font(source)?;
                }
            }
        }

        Ok(())
    }

    /// Loads the system fonts.
    ///
    /// This is a platform-specific operation, for more information see the
    /// documentation for [`fontdb::Database::load_system_fonts`](cosmic_text::fontdb::Database::load_system_fonts).
    pub fn load_system_fonts(&mut self) {
        self.font_system.db_mut().load_system_fonts();
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

    /// Prepare a text buffer for rasterization.
    pub fn prepare_text(&mut self, buffer: &Buffer, scale: f32) {
        loop {
            if self.try_prepare_text(buffer, scale) {
                break;
            }

            self.font_atlas.grow();
        }
    }

    fn try_prepare_text(&mut self, buffer: &Buffer, scale: f32) -> bool {
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let rasterized = self.font_atlas.rasterize_glyph(
                    &mut self.swash_cache,
                    &mut self.font_system,
                    glyph,
                    scale,
                );

                if rasterized.is_none() {
                    return false;
                }
            }
        }

        true
    }

    /// Convert a text buffer to a mesh.
    ///
    /// This involves shapind the text, rasterizing the glyphs, laying out the glyphs,
    /// and creating the mesh itself, and should ideally be done as little as possible.
    pub fn rasterize_text(&mut self, buffer: &Buffer, scale: f32) -> Mesh {
        // if rasterizing returns None, it means the font atlas is full
        // so we need to grow it and try again
        //
        // TODO: handle the case where the font atlas is full and we can't grow it
        loop {
            if let Some(mesh) = self.try_rasterize_text(buffer, scale) {
                break mesh;
            }

            self.font_atlas.grow();
        }
    }

    fn try_rasterize_text(&mut self, buffer: &Buffer, scale: f32) -> Option<Mesh> {
        fn round_point(point: Point, scale: f32) -> Point {
            Point::round(point * scale) / scale
        }

        let mut mesh = Mesh::new();
        let mut glyphs = Vec::<(Rect, Rect, Color)>::new();

        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let rasterized = self.font_atlas.rasterize_glyph(
                    &mut self.swash_cache,
                    &mut self.font_system,
                    glyph,
                    scale,
                )?;

                let physical = glyph.physical((0.0, 0.0), 1.0);

                let min = Point::new(physical.x as f32, run.line_y + physical.y as f32);
                let rect = Rect::min_size(min + rasterized.offset, rasterized.size);

                let color = match glyph.color_opt {
                    Some(color) => Color::rgba(
                        color.r() as f32 / 255.0,
                        color.g() as f32 / 255.0,
                        color.b() as f32 / 255.0,
                        color.a() as f32 / 255.0,
                    ),
                    None => Color::BLACK,
                };

                glyphs.push((rasterized.uv, rect, color));
            }
        }

        for (uv, rect, color) in glyphs {
            let index = mesh.vertices.len() as u32;

            mesh.vertices.push(Vertex {
                position: round_point(rect.top_left(), scale),
                tex_coords: uv.top_left(),
                color,
            });
            mesh.vertices.push(Vertex {
                position: round_point(rect.top_right(), scale),
                tex_coords: uv.top_right(),
                color,
            });
            mesh.vertices.push(Vertex {
                position: round_point(rect.bottom_right(), scale),
                tex_coords: uv.bottom_right(),
                color,
            });
            mesh.vertices.push(Vertex {
                position: round_point(rect.bottom_left(), scale),
                tex_coords: uv.bottom_left(),
                color,
            });

            mesh.indices.push(index);
            mesh.indices.push(index + 1);
            mesh.indices.push(index + 2);

            mesh.indices.push(index);
            mesh.indices.push(index + 2);
            mesh.indices.push(index + 3);
        }

        mesh.set_texture(self.font_atlas.image().clone());

        Some(mesh)
    }
}
