use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use smallvec::SmallVec;
use smol_str::{format_smolstr, SmolStr};

use super::{FontAttributes, TextAlign, TextWrap};

/// A paragraph of rich text, that can contain multiple segments with different [`FontAttributes`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Paragraph {
    /// The line height of the text.
    pub line_height: f32,

    /// The alignment of the text.
    pub align: TextAlign,

    /// The text wrapping mode.
    pub wrap: TextWrap,

    text: SmolStr,
    segments: SmallVec<[Segment; 1]>,
}

impl Paragraph {
    /// Create a new empty paragraph.
    pub fn new(line_height: f32, align: TextAlign, wrap: TextWrap) -> Self {
        Self {
            line_height,
            align,
            wrap,
            text: SmolStr::default(),
            segments: SmallVec::new(),
        }
    }

    /// Clear the paragraph.
    pub fn clear(&mut self) {
        self.text = SmolStr::default();
        self.segments.clear();
    }

    /// Set the text of the paragraph with the given [`FontAttributes`].
    pub fn set_text(&mut self, text: impl Display, attrs: FontAttributes) {
        self.clear();
        self.push_text(text, attrs);
    }

    /// Push a new segment of text with the given [`FontAttributes`] to the paragraph.
    pub fn push_text(&mut self, text: impl Display, attrs: FontAttributes) {
        self.text = format_smolstr!("{}{}", self.text, text);
        self.segments.push(Segment {
            end: self.text.len(),
            attrs,
        });
    }

    /// Push a new segment of text with the given [`FontAttributes`] to the paragraph.
    pub fn with_text(mut self, text: impl Display, attrs: FontAttributes) -> Self {
        self.push_text(text, attrs);
        self
    }

    /// Get the text of the paragraph.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get an iterator over the segments of the paragraph.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (&str, &FontAttributes)> {
        self.segments.iter().map(|segment| {
            let start = segment.end - self.text.len();
            let end = segment.end;
            let text = &self.text[start..end];
            (text, &segment.attrs)
        })
    }

    /// Get an iterator over the segments of the paragraph mutably.
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = (&str, &mut FontAttributes)> {
        let text = &self.text;

        self.segments.iter_mut().map(|segment| {
            let start = segment.end - self.text.len();
            let end = segment.end;
            let text = &text[start..end];
            (text, &mut segment.attrs)
        })
    }
}

impl Eq for Paragraph {}

impl Hash for Paragraph {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.line_height.to_bits().hash(state);
        self.align.hash(state);
        self.wrap.hash(state);
        self.text.hash(state);
        self.segments.hash(state);
    }
}

/// A segment of a [`Paragraph`], that starts at the end of the previous segment, or at index 0,
/// and ends at [`Segment::end`]. The purpose of a segment is to specify the text attributes for
/// the range of the text that the segment covers.
#[derive(Clone, Debug, PartialEq, Hash)]
struct Segment {
    end: usize,
    attrs: FontAttributes,
}
