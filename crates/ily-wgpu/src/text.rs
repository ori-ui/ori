use ily_graphics::{TextAlign, TextSection};

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
        let aligned_rect = section.aligned_rect();
        let x = aligned_rect.min.x;
        let y = aligned_rect.min.y;
        let width = aligned_rect.width();
        let height = aligned_rect.height();

        let font_id = if let Some(font) = &section.font {
            self.find_font(font)
        } else {
            wgpu_glyph::FontId::default()
        };

        let text = wgpu_glyph::Text {
            text: &section.text,
            scale: section.scale.into(),
            font_id,
            extra: wgpu_glyph::Extra {
                color: section.color.into(),
                z: 1.0,
            },
        };

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
