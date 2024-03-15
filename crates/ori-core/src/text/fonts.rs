use std::io;

use cosmic_text::{Buffer, FontSystem, SwashCache};

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
        let font_system = FontSystem::new();
        let font_atlas = FontAtlas::new();

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
        let db = self.font_system.db_mut();

        db.load_font_data(include_bytes!("../../font/NotoSans-Regular.ttf").to_vec());
        db.load_system_fonts();
        db.set_sans_serif_family("Noto Sans");
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

        Size::new(width, height).round()
    }

    /// Convert a text buffer to a mesh.
    ///
    /// This involves shapind the text, rasterizing the glyphs, laying out the glyphs,
    /// and creating the mesh itself, and should ideally be done as little as possible.
    pub fn rasterize_text(&mut self, buffer: &Buffer) -> Mesh {
        // if rasterizing returns None, it means the font atlas is full
        // so we need to grow it and try again
        //
        // TODO: handle the case where the font atlas is full and we can't grow it
        loop {
            if let Some(mesh) = self.try_rasterize_text(buffer) {
                break mesh;
            }

            self.font_atlas.grow();
        }
    }

    fn try_rasterize_text(&mut self, buffer: &Buffer) -> Option<Mesh> {
        let mut mesh = Mesh::new();
        let mut glyphs = Vec::<(Rect, Rect, Color)>::new();

        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let rasterized = self.font_atlas.rasterize_glyph(
                    &mut self.swash_cache,
                    &mut self.font_system,
                    glyph,
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
                position: rect.top_left(),
                tex_coords: uv.top_left(),
                color,
            });
            mesh.vertices.push(Vertex {
                position: rect.top_right(),
                tex_coords: uv.top_right(),
                color,
            });
            mesh.vertices.push(Vertex {
                position: rect.bottom_right(),
                tex_coords: uv.bottom_right(),
                color,
            });
            mesh.vertices.push(Vertex {
                position: rect.bottom_left(),
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
