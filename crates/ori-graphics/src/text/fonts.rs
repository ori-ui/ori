use std::{collections::HashMap, io, path::Path, sync::Arc};

use fontdue::{
    layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle},
    Font, FontSettings,
};
use glam::Vec2;

use crate::{FontAtlas, FontQuery, Mesh, Rect, Renderer, TextSection, Vertex};

/// An error that occurred while loading fonts.
#[derive(Debug)]
pub enum FontsError {
    /// An I/O error occurred.
    Io(io::Error),
    /// A fontdue error occurred.
    Fontdue(&'static str),
}

impl From<io::Error> for FontsError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<&'static str> for FontsError {
    fn from(err: &'static str) -> Self {
        Self::Fontdue(err)
    }
}

/// A collection of loaded fonts.
#[derive(Clone, Debug, Default)]
pub struct Fonts {
    db: fontdb::Database,
    font_cache: HashMap<fontdb::ID, Option<Arc<Font>>>,
    font_atlases: HashMap<fontdb::ID, FontAtlas>,
    query_cache: HashMap<FontQuery, fontdb::ID>,
}

impl Fonts {
    /// Creates a new font collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads a font from `data`.
    pub fn load_font_data(&mut self, data: Vec<u8>) {
        self.db.load_font_data(data);
    }

    /// Loads a font from a file.
    pub fn load_font_file(&mut self, path: impl AsRef<Path>) -> Result<(), FontsError> {
        self.db.load_font_file(path)?;
        Ok(())
    }

    pub fn load_fonts_dir(&mut self, path: impl AsRef<Path>) {
        self.db.load_fonts_dir(path);
    }

    /// Loads the system fonts.
    pub fn load_system_fonts(&mut self) {
        self.db.load_system_fonts();
        self.db.set_serif_family("Noto Serif");
        self.db.set_sans_serif_family("Noto Sans");
        self.db.set_monospace_family("Noto Sans Mono");
        self.db.set_cursive_family("Comic Sans MS");
        self.db.set_fantasy_family("Impact");
    }

    /// Queries the font collection for a font matching `query`.
    pub fn query_id(&mut self, query: &FontQuery) -> Option<fontdb::ID> {
        if let Some(id) = self.query_cache.get(query) {
            return Some(*id);
        }

        let fontdb_query = fontdb::Query {
            families: &[query.family.to_fontdb()],
            weight: query.weight.to_fontdb(),
            stretch: query.stretch.to_fontdb(),
            style: query.style.to_fontdb(),
        };

        let id = self.db.query(&fontdb_query)?;

        self.query_cache.insert(query.clone(), id);

        Some(id)
    }

    /// Queries the font collection for a font matching `query`.
    pub fn query(&mut self, query: &FontQuery) -> Option<Arc<Font>> {
        let id = self.query_id(query)?;
        self.get_font(id)
    }

    /// Gets a font from the font collection.
    pub fn get_font(&mut self, id: fontdb::ID) -> Option<Arc<Font>> {
        if let Some(font) = self.font_cache.get(&id) {
            return font.clone();
        }

        let font = self.db.with_face_data(id, |data, index| {
            let settings = FontSettings {
                scale: 80.0,
                collection_index: index,
            };

            Font::from_bytes(data, settings)
        });
        let font = Arc::new(font?.ok()?);

        self.font_cache.insert(id, Some(font.clone()));

        Some(font)
    }

    /// Queries the font collection for a font atlas matching `query`.
    pub fn query_atlas(&mut self, query: &FontQuery) -> Option<&mut FontAtlas> {
        let id = self.query_id(query)?;

        if self.font_atlases.contains_key(&id) {
            return self.font_atlases.get_mut(&id);
        }

        let atlas = FontAtlas::new();
        Some(self.font_atlases.entry(id).or_insert(atlas))
    }

    /// Gets a font atlas from the font collection.
    pub fn get_atlas(&mut self, id: fontdb::ID) -> &mut FontAtlas {
        self.font_atlases.entry(id).or_insert_with(FontAtlas::new)
    }

    fn text_layout_inner(&mut self, font: &Font, text: &TextSection<'_>) -> Option<Layout> {
        let settings = LayoutSettings {
            x: text.rect.min.x,
            y: text.rect.min.y,
            max_width: Some(text.rect.width()),
            max_height: Some(text.rect.height()),
            horizontal_align: text.h_align.to_horizontal(),
            vertical_align: text.v_align.to_vertical(),
            line_height: text.line_height,
            wrap_style: text.wrap.to_fontdue(),
            wrap_hard_breaks: true,
        };

        let text_style = TextStyle {
            text: text.text,
            px: text.font_size,
            font_index: 0,
            user_data: (),
        };

        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&settings);
        layout.append(&[font], &text_style);

        Some(layout)
    }

    pub fn text_layout(&mut self, text: &TextSection<'_>) -> Option<Layout> {
        let id = self.query_id(&text.font_query())?;
        let font = self.get_font(id)?;

        self.text_layout_inner(&font, text)
    }

    pub fn measure_text(&mut self, text: &TextSection<'_>) -> Option<Rect> {
        let layout = self.text_layout(text)?;

        if layout.glyphs().is_empty() {
            return None;
        }

        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = f32::NEG_INFINITY;

        for glyph in layout.glyphs() {
            let position = Vec2::new(glyph.x, glyph.y);
            min = Vec2::min(min, position);
            max = f32::max(max, position.x + glyph.width as f32);
        }

        let width = max - min.x;
        let height = layout.height();
        let rect = Rect::min_size(min, Vec2::new(width, height));
        let rect = rect.expand(Vec2::new(0.2, 0.0) * text.font_size);

        Some(rect)
    }

    pub fn text_mesh(&mut self, renderer: &dyn Renderer, text: &TextSection<'_>) -> Option<Mesh> {
        let id = self.query_id(&text.font_query())?;
        let font = self.get_font(id)?;
        let layout = self.text_layout_inner(&font, text)?;
        let atlas = self.get_atlas(id);

        let mut glyphs = Vec::<Rect>::new();

        'outer: loop {
            for glyph in layout.glyphs() {
                match atlas.glyph_rect_uv(renderer, &font, glyph.key) {
                    Some(rect) => glyphs.push(rect),
                    None => {
                        atlas.grow(renderer);
                        continue 'outer;
                    }
                }
            }

            break;
        }

        let mut mesh = Mesh::new();

        for (glyph, uv) in layout.glyphs().iter().zip(glyphs) {
            let min = Vec2::new(glyph.x, glyph.y);
            let size = Vec2::new(glyph.width as f32, glyph.height as f32);
            let glyph_rect = Rect::min_size(min, size);

            let index = mesh.vertices.len() as u32;

            mesh.vertices.push(Vertex {
                position: glyph_rect.top_left(),
                uv: uv.top_left(),
                color: text.color,
            });
            mesh.vertices.push(Vertex {
                position: glyph_rect.top_right(),
                uv: uv.top_right(),
                color: text.color,
            });
            mesh.vertices.push(Vertex {
                position: glyph_rect.bottom_right(),
                uv: uv.bottom_right(),
                color: text.color,
            });
            mesh.vertices.push(Vertex {
                position: glyph_rect.bottom_left(),
                uv: uv.bottom_left(),
                color: text.color,
            });

            mesh.indices.push(index);
            mesh.indices.push(index + 1);
            mesh.indices.push(index + 2);
            mesh.indices.push(index);
            mesh.indices.push(index + 2);
            mesh.indices.push(index + 3);
        }

        mesh.image = atlas.image().cloned();

        Some(mesh)
    }
}
