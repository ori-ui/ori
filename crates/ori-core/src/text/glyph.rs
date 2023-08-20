use std::{ops::Deref, vec};

use fontdue::layout::GlyphRasterConfig;
use glam::Vec2;

use crate::{
    canvas::Color,
    layout::{Rect, Size},
};

use super::{TextAlign, TextWrap};

/// A laid out glyph.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
    /// The character of the glyph.
    pub code: char,
    /// The rect of the glyph.
    pub rect: Rect,
    /// The byte offset of the glyph.
    pub byte_offset: usize,
    /// The line of the glyph.
    pub line: usize,
    /// The baseline of the glyph.
    pub baseline: f32,
    /// The decent of the glyph.
    pub line_descent: f32,
    /// The ascent of the glyph.
    pub line_ascent: f32,
    /// The advance of the glyph.
    pub advance: f32,
    /// The key of the glyph.
    pub key: GlyphRasterConfig,
}

/// A collection of laid out glyphs.
#[derive(Clone, Debug, Default)]
pub struct Glyphs {
    pub(crate) glyphs: Vec<Glyph>,
    pub(crate) size: Size,
    pub(crate) font: fontdb::ID,
    pub(crate) wrap: TextWrap,
    pub(crate) h_align: TextAlign,
    pub(crate) v_align: TextAlign,
    pub(crate) color: Color,
}

impl Glyphs {
    /// Get the size of the text as a whole.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Get the font id.
    pub fn font(&self) -> fontdb::ID {
        self.font
    }

    /// Get the text wrap.
    pub fn wrap(&self) -> TextWrap {
        self.wrap
    }

    /// Get the horizontal alignment.
    pub fn h_align(&self) -> TextAlign {
        self.h_align
    }

    /// Get the vertical alignment.
    pub fn v_align(&self) -> TextAlign {
        self.v_align
    }

    /// Compute the offset of the text in a `rect`.
    pub fn offset(&self, rect: Rect) -> Vec2 {
        let x_diff = rect.width() - self.size().width;
        let y_diff = rect.height() - self.size().height;

        let x_offset = if self.wrap() != TextWrap::None {
            match self.h_align() {
                TextAlign::Left => 0.0,
                TextAlign::Center => x_diff / 2.0,
                TextAlign::Right => x_diff,
            }
        } else {
            0.0
        };
        let y_offset = if self.wrap() != TextWrap::None {
            match self.v_align() {
                TextAlign::Top => 0.0,
                TextAlign::Center => y_diff / 2.0,
                TextAlign::Bottom => y_diff,
            }
        } else {
            0.0
        };

        Vec2::new(x_offset, y_offset)
    }

    /// Get the color.
    pub fn color(&self) -> Color {
        self.color
    }
}

impl Deref for Glyphs {
    type Target = [Glyph];

    fn deref(&self) -> &Self::Target {
        &self.glyphs
    }
}

impl IntoIterator for Glyphs {
    type Item = Glyph;
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.glyphs.into_iter()
    }
}

impl<'a> IntoIterator for &'a Glyphs {
    type Item = &'a Glyph;
    type IntoIter = std::slice::Iter<'a, Glyph>;

    fn into_iter(self) -> Self::IntoIter {
        self.glyphs.iter()
    }
}
