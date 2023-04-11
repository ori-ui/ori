use std::{cell::RefCell, collections::HashMap};

use ily_core::Vec2;
use ily_graphics::{Rect, TextSection};

#[derive(Default)]
pub struct Fonts {
    pub fonts: Vec<String>,
}

impl Fonts {
    pub fn find_font(&self, name: &str) -> Option<wgpu_glyph::FontId> {
        let mut fonts = HashMap::new();

        for (i, font) in self.fonts.iter().enumerate() {
            fonts.insert(font.as_str(), wgpu_glyph::FontId(i));
        }

        fonts.get(name).copied()
    }

    pub fn convert_section<'a>(&'a self, section: &'a TextSection) -> wgpu_glyph::Section<'a> {
        let x = section.bounds.min.x;
        let y = section.bounds.min.y;
        let width = section.bounds.size().x;
        let height = section.bounds.size().y;

        let mut text = wgpu_glyph::Text::new(&section.text)
            .with_color(section.color)
            .with_scale(section.scale);

        if let Some(ref font) = section.font {
            if let Some(font) = self.find_font(font) {
                text = text.with_font_id(font);
            }
        }

        wgpu_glyph::Section {
            screen_position: (x, y),
            bounds: (width, height),
            layout: wgpu_glyph::Layout::default(),
            text: vec![text],
        }
    }
}

pub struct TextLayout<'a, T: wgpu_glyph::GlyphCruncher> {
    pub fonts: &'a Fonts,
    pub glyph: &'a RefCell<T>,
}

impl<'a, T: wgpu_glyph::GlyphCruncher> ily_graphics::TextLayout for TextLayout<'a, T> {
    fn bounds(&self, section: &TextSection) -> Option<Rect> {
        let section = self.fonts.convert_section(section);
        let bounds = self.glyph.borrow_mut().glyph_bounds(section)?;

        Some(Rect {
            min: Vec2::new(bounds.min.x, bounds.min.y),
            max: Vec2::new(bounds.max.x, bounds.max.y),
        })
    }
}
