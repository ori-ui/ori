use std::cell::RefCell;

use ily_core::Vec2;
use ily_graphics::{Rect, TextAlign, TextHit, TextSection};
use wgpu_glyph::ab_glyph::{Font, ScaleFont};

fn convert_h_align(align: TextAlign) -> wgpu_glyph::HorizontalAlign {
    match align {
        TextAlign::Start => wgpu_glyph::HorizontalAlign::Left,
        TextAlign::Center => wgpu_glyph::HorizontalAlign::Center,
        TextAlign::End => wgpu_glyph::HorizontalAlign::Right,
    }
}

fn convert_v_align(align: TextAlign) -> wgpu_glyph::VerticalAlign {
    match align {
        TextAlign::Start => wgpu_glyph::VerticalAlign::Top,
        TextAlign::Center => wgpu_glyph::VerticalAlign::Center,
        TextAlign::End => wgpu_glyph::VerticalAlign::Bottom,
    }
}

#[derive(Default)]
pub struct Fonts {
    pub fonts: Vec<String>,
}

impl Fonts {
    pub fn add_font(&mut self, font: impl Into<String>) {
        self.fonts.push(font.into());
    }

    pub fn find_font(&self, name: &str) -> wgpu_glyph::FontId {
        for (i, font) in self.fonts.iter().enumerate() {
            if font == name {
                return wgpu_glyph::FontId(i);
            }
        }

        wgpu_glyph::FontId::default()
    }

    pub fn convert_section<'a>(&'a self, section: &'a TextSection) -> wgpu_glyph::Section<'a> {
        let x = section.position.x;
        let y = section.position.y;
        let width = section.bounds.x;
        let height = section.bounds.y;

        let mut text = wgpu_glyph::Text::new(&section.text)
            .with_color(section.color)
            .with_scale(section.scale);

        if let Some(font) = &section.font {
            text = text.with_font_id(self.find_font(font));
        }

        let layout = if section.wrap {
            wgpu_glyph::Layout::Wrap {
                line_breaker: Default::default(),
                h_align: convert_h_align(section.h_align),
                v_align: convert_v_align(section.v_align),
            }
        } else {
            wgpu_glyph::Layout::SingleLine {
                line_breaker: Default::default(),
                h_align: convert_h_align(section.h_align),
                v_align: convert_v_align(section.v_align),
            }
        };

        wgpu_glyph::Section {
            screen_position: (x, y),
            bounds: (width, height),
            layout,
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

    fn hit(&self, section: &TextSection, postition: Vec2) -> Option<TextHit> {
        let glyph = self.glyph.borrow_mut();
        let font_id = if let Some(font) = &section.font {
            self.fonts.find_font(font)
        } else {
            wgpu_glyph::FontId::default()
        };

        let font = glyph.fonts()[font_id.0].clone().into_scaled(section.scale);
        let section = self.fonts.convert_section(section);

        let mut closest_distance = f32::INFINITY;
        let mut closest = None;

        for glyph in self.glyph.borrow_mut().glyphs(section) {
            let wgpu_glyph::SectionGlyph {
                ref glyph,
                byte_index,
                ..
            } = *glyph;

            let min = Vec2::new(
                glyph.position.x - font.h_side_bearing(glyph.id),
                glyph.position.y - font.ascent(),
            );
            let size = Vec2::new(font.h_advance(glyph.id), font.ascent() - font.descent());

            let rect = Rect::min_size(min, size);

            if rect.contains(postition) {
                return Some(TextHit::Inside(byte_index));
            } else {
                let distance = rect.center().distance(postition);

                if distance < closest_distance {
                    closest_distance = distance;
                    closest = Some(byte_index);
                }
            }
        }

        if let Some(byte_index) = closest {
            Some(TextHit::Outside(byte_index))
        } else {
            None
        }
    }
}
