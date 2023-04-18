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
        let width = aligned_rect.width() + 5.0;
        let height = aligned_rect.height() + 5.0;

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
