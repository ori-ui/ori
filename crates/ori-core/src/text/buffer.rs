use cosmic_text::{Buffer, Shaping};

use crate::layout::{Point, Rect, Size};

use super::{Fonts, TextAlign, TextAttributes, TextWrap};

/// A convenient wrapper around a [`cosmic_text::Buffer`].
#[derive(Debug)]
pub struct TextBuffer {
    buffer: Buffer,
}

impl TextBuffer {
    /// Create a new text buffer.
    pub fn new(fonts: &mut Fonts, font_size: f32, line_height: f32) -> Self {
        let buffer = Buffer::new(
            &mut fonts.font_system,
            cosmic_text::Metrics {
                font_size,
                line_height: line_height * font_size,
            },
        );

        Self { buffer }
    }

    /// Create a new text buffer from a raw buffer.
    pub fn from_raw(buffer: Buffer) -> Self {
        Self { buffer }
    }

    /// Get the raw buffer.
    pub fn raw(&self) -> &Buffer {
        &self.buffer
    }

    /// Get the raw buffer mutably.
    pub fn raw_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    /// Set the metrics of the text buffer.
    pub fn set_metrics(&mut self, fonts: &mut Fonts, font_size: f32, line_height: f32) {
        self.buffer.set_metrics(
            &mut fonts.font_system,
            cosmic_text::Metrics {
                font_size,
                line_height: line_height * font_size,
            },
        );
    }

    /// Set the align of the text buffer.
    pub fn set_align(&mut self, align: TextAlign) {
        for line in &mut self.buffer.lines {
            line.set_align(Some(align.to_cosmic_text()));
        }
    }

    /// Get the size of the text buffer.
    pub fn size(&self) -> Size {
        Fonts::buffer_size(&self.buffer)
    }

    /// Get the rect of the text buffer.
    pub fn rect(&self) -> Rect {
        Rect::min_size(Point::ZERO, self.size())
    }

    /// Get the bounds of the text buffer.
    pub fn bounds(&self) -> Size {
        let (width, height) = self.buffer.size();
        Size::new(width, height)
    }

    /// Set the bounds of the text buffer.
    pub fn set_bounds(&mut self, fonts: &mut Fonts, bounds: Size) {
        (self.buffer).set_size(&mut fonts.font_system, bounds.width, bounds.height);
    }

    /// Set the text of the text buffer.
    pub fn set_text(&mut self, fonts: &mut Fonts, text: &str, attrs: TextAttributes) {
        self.buffer.set_text(
            &mut fonts.font_system,
            text,
            attrs.to_cosmic_text(),
            Shaping::Advanced,
        );
    }

    /// Set the wrapping mode of the text buffer.
    pub fn set_wrap(&mut self, fonts: &mut Fonts, wrap: TextWrap) {
        (self.buffer).set_wrap(&mut fonts.font_system, wrap.to_cosmic_text());
    }
}

impl AsRef<Buffer> for TextBuffer {
    fn as_ref(&self) -> &Buffer {
        &self.buffer
    }
}

impl AsMut<Buffer> for TextBuffer {
    fn as_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }
}
