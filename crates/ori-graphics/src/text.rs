use std::{fmt::Display, str::FromStr};

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Stretch, Style, Weight, Wrap};
use glam::Vec2;

use crate::{Color, Rect};

/// Alignment of text within its bounds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAlign {
    /// Align text to the left, or top.
    #[default]
    Start,
    /// Align text to the center.
    Center,
    /// Align text to the right, or bottom.
    End,
}

impl Display for TextAlign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextAlign::Start => write!(f, "start"),
            TextAlign::Center => write!(f, "center"),
            TextAlign::End => write!(f, "end"),
        }
    }
}

impl FromStr for TextAlign {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left" | "start" => Ok(TextAlign::Start),
            "center" => Ok(TextAlign::Center),
            "right" | "end" => Ok(TextAlign::End),
            _ => Err(()),
        }
    }
}

impl TextAlign {
    /// Align a point within a range.
    pub fn align(&self, start: f32, end: f32) -> f32 {
        match self {
            TextAlign::Start => start,
            TextAlign::Center => (start + end) / 2.0,
            TextAlign::End => end,
        }
    }
}

/// A section of text to display.
///
/// Text is rendered using the [`cosmic_text`] crate.
#[derive(Clone, Debug)]
pub struct TextSection {
    /// The rectangle to display the text in.
    pub rect: Rect,
    /// The scale of the text in pixels.
    pub scale: f32,
    /// Horizontal alignment of the text.
    pub h_align: TextAlign,
    /// Vertical alignment of the text.
    pub v_align: TextAlign,
    /// Whether to wrap the text.
    pub wrap: bool,
    /// The text to display.
    pub text: String,
    /// The font family to use.
    pub font_family: Option<String>,
    /// The color of the text.
    pub color: Color,
}

impl Default for TextSection {
    fn default() -> Self {
        Self {
            rect: Rect::new(Vec2::ZERO, Vec2::splat(f32::INFINITY)),
            scale: 16.0,
            h_align: TextAlign::Start,
            v_align: TextAlign::Start,
            wrap: true,
            text: String::new(),
            font_family: None,
            color: Color::BLACK,
        }
    }
}

impl TextSection {
    /// Creates a [`Buffer`] for this section of text.
    pub fn buffer(&self, font_system: &mut FontSystem) -> Buffer {
        let metrics = Metrics {
            font_size: self.scale,
            line_height: self.scale,
        };

        let family = match self.font_family {
            Some(ref name) => Family::Name(&name),
            None => Family::SansSerif,
        };

        let attrs = Attrs {
            color_opt: Some(cosmic_text::Color::rgba(
                (self.color.r * 255.0) as u8,
                (self.color.g * 255.0) as u8,
                (self.color.b * 255.0) as u8,
                (self.color.a * 255.0) as u8,
            )),
            family,
            stretch: Stretch::Normal,
            style: Style::Normal,
            weight: Weight::NORMAL,
            metadata: 0,
        };

        let mut buffer = Buffer::new(font_system, metrics);
        buffer.set_size(font_system, self.rect.width(), f32::INFINITY);
        buffer.set_text(font_system, &self.text, attrs);

        let wrap = if self.wrap { Wrap::Word } else { Wrap::None };
        buffer.set_wrap(font_system, wrap);

        buffer
    }

    /// Messures the bounds of a [`Buffer`] for this section of text.
    pub fn messure_buffer(&self, font_system: &mut FontSystem, buffer: &Buffer) -> Rect {
        // TODO: i have no idea what this is doing
        // this is just a copy paste from
        //
        // https://github.com/iced-rs/iced/blob/master/wgpu/src/text.rs
        let (total_lines, max_with) = buffer
            .layout_runs()
            .enumerate()
            .fold((0, 0.0), |(_, max), (i, buffer)| {
                (i + 1, buffer.line_w.max(max))
            });

        let total_height = total_lines as f32 * buffer.metrics().line_height;

        // here we're getting the font from the first glyph in the first line
        // and then getting the descender from that font to calculate the descent
        // and offsetting the text by that amount
        //
        // this shouldn't be necessary, but it is, due to a bug in cosmic-text
        // https://github.com/pop-os/cosmic-text/issues/123
        let font = if let Some(line) = buffer.layout_runs().next() {
            (line.glyphs.get(0)).and_then(|g| font_system.get_font(g.cache_key.font_id))
        } else {
            None
        };

        let descent = if let Some(font) = font {
            let descender = font.rustybuzz().descender();
            let units_per_em = font.rustybuzz().units_per_em();

            let scale = buffer.metrics().font_size / units_per_em as f32;
            descender as f32 * scale
        } else {
            0.0
        };

        Rect {
            min: self.rect.min + Vec2::new(0.0, descent),
            max: self.rect.min + Vec2::new(max_with, total_height),
        }
    }

    /// Measures the bounds of this section of text.
    pub fn measure(&self, font_system: &mut FontSystem) -> Rect {
        let buffer = self.buffer(font_system);
        self.messure_buffer(font_system, &buffer)
    }
}

/// A laid out glyph in a [`TextSection`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
    /// The byte index of the glyph in the text.
    pub index: usize,
    /// The byte index of the next glyph in the text.
    pub rect: Rect,
}

/// A laid out line in a [`TextSection`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Line {
    /// The byte index of the first glyph in the line.
    pub index: usize,
    /// The [`Glyph`]s in the line.
    pub glyphs: Vec<Glyph>,
    /// The bounds of the line.
    pub rect: Rect,
}

/// A hit in a [`TextSection`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextHit {
    /// The byte index of the hit.
    pub index: usize,
    /// Whether the hit is inside a glyph.
    pub inside: bool,
    /// The delta between the hit and the center of the glyph.
    pub delta: Vec2,
}
