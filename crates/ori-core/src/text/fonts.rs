use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use fontdue::{
    layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle},
    Font, FontSettings, Metrics,
};
use glam::Vec2;

use crate::{
    FontAtlas, FontQuery, Glyph, Glyphs, Mesh, Rect, Size, TextAlign, TextSection, TextWrap, Vertex,
};

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

/// A source for a font.
#[derive(Clone, Debug)]
pub enum FontSource {
    /// A font loaded from data.
    Data(Vec<u8>),
    /// A font loaded from a file.
    Path(PathBuf),
    /// A collection of fonts.
    Set(Vec<FontSource>),
}

impl From<Vec<u8>> for FontSource {
    fn from(data: Vec<u8>) -> Self {
        Self::Data(data)
    }
}

impl From<&[u8]> for FontSource {
    fn from(data: &[u8]) -> Self {
        Self::Data(data.to_vec())
    }
}

impl From<&str> for FontSource {
    fn from(data: &str) -> Self {
        Self::Data(data.as_bytes().to_vec())
    }
}

impl From<&Path> for FontSource {
    fn from(path: &Path) -> Self {
        Self::Path(path.to_path_buf())
    }
}

impl From<PathBuf> for FontSource {
    fn from(path: PathBuf) -> Self {
        Self::Path(path)
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

    /// Loads a font from `source`.
    pub fn load_font(&mut self, source: impl Into<FontSource>) -> Result<(), FontsError> {
        let source = source.into();
        match source {
            FontSource::Data(data) => {
                self.load_font_data(data);
                Ok(())
            }
            FontSource::Set(sources) => {
                for source in sources {
                    self.load_font(source)?;
                }

                Ok(())
            }
            FontSource::Path(path) if path.is_dir() => {
                self.load_fonts_dir(path);
                Ok(())
            }
            FontSource::Path(path) => {
                self.load_font_file(path)?;
                Ok(())
            }
        }
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

    /// Loads all fonts from a directory.
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
        let max_width = match text.wrap {
            TextWrap::None => None,
            _ => Some(text.bounds.width),
        };

        let max_height = match text.wrap {
            TextWrap::None => Some(text.bounds.width),
            _ => None,
        };

        let settings = LayoutSettings {
            x: 0.0,
            y: 0.0,
            max_width,
            max_height,
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

    /// Creates a text layout for `text` and returns the glyphs.
    pub fn layout_text(&mut self, text: &TextSection<'_>) -> Option<Glyphs> {
        let id = self.query_id(&text.font_query())?;
        let font = self.get_font(id)?;

        let layout = self.text_layout_inner(&font, text)?;
        let size = self.measure_layout(&font, &layout, text)?;

        let x_diff = size.width - text.bounds.width;
        let x_offset = if text.wrap != TextWrap::None {
            match text.h_align {
                TextAlign::Left => 0.0,
                TextAlign::Center => x_diff / 2.0,
                TextAlign::Right => x_diff,
            }
        } else {
            0.0
        };

        let y_diff = size.height - text.bounds.height;
        let y_offset = if text.wrap == TextWrap::None {
            match text.v_align {
                TextAlign::Top => 0.0,
                TextAlign::Center => y_diff / 2.0,
                TextAlign::Bottom => y_diff,
            }
        } else {
            0.0
        };

        let offset = Vec2::new(x_offset, y_offset);
        let glyphs = self.layout_glyphs_inner(&font, &layout, offset)?;

        Some(crate::Glyphs {
            glyphs,
            size,
            font: id,
            wrap: text.wrap,
            h_align: text.h_align,
            v_align: text.v_align,
            color: text.color,
        })
    }

    fn layout_glyphs_inner(
        &self,
        font: &Font,
        layout: &Layout,
        offset: Vec2,
    ) -> Option<Vec<Glyph>> {
        if layout.glyphs().is_empty() {
            return None;
        }

        let mut glyphs = Vec::new();

        for (line_index, line) in layout.lines().into_iter().flatten().enumerate() {
            if line.glyph_end < line.glyph_start {
                continue;
            }

            for glyph in &layout.glyphs()[line.glyph_start..=line.glyph_end] {
                let metrics = if !glyph.char_data.is_control() {
                    font.metrics(glyph.parent, glyph.key.px)
                } else {
                    Metrics::default()
                };
                let advance = metrics.advance_width;

                let min = Vec2::new(glyph.x, glyph.y) + offset;
                let size = Size::new(metrics.width as f32, metrics.height as f32);

                let glyph = Glyph {
                    code: glyph.parent,
                    rect: Rect::min_size(min, size),
                    byte_offset: glyph.byte_offset,
                    line: line_index,
                    baseline: line.baseline_y + offset.y,
                    line_descent: line.min_descent,
                    line_ascent: line.max_ascent,
                    advance,
                    key: glyph.key,
                };

                glyphs.push(glyph);
            }
        }

        Some(glyphs)
    }

    fn measure_layout(
        &self,
        font: &Font,
        layout: &Layout,
        _text: &TextSection<'_>,
    ) -> Option<Size> {
        if layout.glyphs().is_empty() {
            return None;
        }

        let mut width = 0.0;

        for line in layout.lines().into_iter().flatten() {
            let mut line_width = 0.0;

            if line.glyph_end < line.glyph_start {
                continue;
            }

            for glyph in &layout.glyphs()[line.glyph_start..=line.glyph_end] {
                let metrics = if !glyph.char_data.is_control() {
                    font.metrics(glyph.parent, glyph.key.px)
                } else {
                    Metrics::default()
                };
                let advance = metrics.advance_width.ceil();

                line_width += advance;
            }

            width = f32::max(width, line_width);
        }

        Some(Size::new(width, layout.height().ceil()))
    }

    /// Creates a mesh for `text`.
    pub fn text_mesh(&mut self, glyphs: &Glyphs, rect: Rect) -> Option<Mesh> {
        let font = self.get_font(glyphs.font())?;
        let atlas = self.get_atlas(glyphs.font());

        let mut uvs = Vec::<Rect>::new();

        'outer: loop {
            for glyph in glyphs.iter() {
                match atlas.glyph_rect_uv(&font, glyph.key) {
                    Some(rect) => uvs.push(rect),
                    None => {
                        atlas.grow();
                        uvs.clear();
                        continue 'outer;
                    }
                }
            }

            break;
        }

        let offset = glyphs.offset(rect);
        let mut mesh = Mesh::new();

        for (glyph, uv) in glyphs.iter().zip(uvs) {
            let rect = glyph.rect + rect.min + offset;
            let index = mesh.vertices.len() as u32;

            mesh.vertices.push(Vertex {
                position: rect.top_left(),
                tex_coords: uv.top_left(),
                color: glyphs.color(),
            });
            mesh.vertices.push(Vertex {
                position: rect.top_right(),
                tex_coords: uv.top_right(),
                color: glyphs.color(),
            });
            mesh.vertices.push(Vertex {
                position: rect.bottom_right(),
                tex_coords: uv.bottom_right(),
                color: glyphs.color(),
            });
            mesh.vertices.push(Vertex {
                position: rect.bottom_left(),
                tex_coords: uv.bottom_left(),
                color: glyphs.color(),
            });

            mesh.indices.push(index);
            mesh.indices.push(index + 1);
            mesh.indices.push(index + 2);
            mesh.indices.push(index);
            mesh.indices.push(index + 2);
            mesh.indices.push(index + 3);
        }

        mesh.image = Some(atlas.image().clone());

        Some(mesh)
    }
}
