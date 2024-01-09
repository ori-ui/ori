use std::io;

use cosmic_text::{Buffer, FontSystem, SwashCache};

use crate::{
    canvas::{Color, Mesh, Vertex},
    layout::{Point, Rect, Size, Vector},
};

use super::{FontAtlas, FontSource};

/// A collection of loaded fonts.
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
    /// Creates a new font collection.
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

    /// Loads a font from a file.
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
    pub fn load_system_fonts(&mut self) {
        self.font_system.db_mut().load_system_fonts();
    }

    /// Get the size of a text buffer.
    pub fn buffer_size(buffer: &Buffer) -> Size {
        let mut width = 0.0;
        let mut height = 0.0;

        for run in buffer.layout_runs() {
            width = f32::max(width, run.line_w);
            height += buffer.metrics().line_height;
        }

        Size::new(width, height)
    }

    /// Convert a text buffer to a mesh.
    pub fn rasterize_text(&mut self, buffer: &Buffer, rect: Rect) -> Mesh {
        loop {
            if let Some(mesh) = self.try_rasterize_text(buffer, rect.min.to_vector()) {
                break mesh;
            }

            self.font_atlas.grow();
        }
    }

    fn try_rasterize_text(&mut self, buffer: &Buffer, offset: Vector) -> Option<Mesh> {
        let mut mesh = Mesh::new();
        let mut glyphs = Vec::<(Rect, Rect, Color)>::new();

        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let rasterized = self.font_atlas.rasterize_glyph(
                    &mut self.swash_cache,
                    &mut self.font_system,
                    glyph,
                )?;

                let physical = glyph.physical((offset.x, offset.y), 1.0);

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
