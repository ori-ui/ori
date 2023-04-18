use std::{fmt::Display, str::FromStr};

use glam::Vec2;

use crate::{Color, Rect};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Start,
    Center,
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
    pub fn align(&self, start: f32, end: f32) -> f32 {
        match self {
            TextAlign::Start => start,
            TextAlign::Center => (start + end) / 2.0,
            TextAlign::End => end,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextSection {
    pub rect: Rect,
    pub scale: f32,
    pub h_align: TextAlign,
    pub v_align: TextAlign,
    pub wrap: bool,
    pub text: String,
    pub font: Option<String>,
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
            font: None,
            color: Color::BLACK,
        }
    }
}

impl TextSection {
    pub fn aligned_rect(&self) -> Rect {
        let x = self.h_align.align(self.rect.min.x, self.rect.max.x);
        let y = self.v_align.align(self.rect.min.y, self.rect.max.y);
        let position = Vec2::new(x, y);

        Rect::min_size(position, self.rect.size())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextHit {
    /// The hit position is inside the text.
    ///
    /// The index is the index of the character that was hit.
    Inside(usize),
    /// The hit position is outside the text.
    ///
    /// The index is the index of the character closest to the hit position.
    Outside(usize),
}

impl TextHit {
    pub const fn index(&self) -> usize {
        match self {
            TextHit::Inside(index) => *index,
            TextHit::Outside(index) => *index,
        }
    }

    pub const fn inside(&self) -> Option<bool> {
        match self {
            TextHit::Inside(_) => Some(true),
            _ => None,
        }
    }

    pub const fn outside(&self) -> Option<bool> {
        match self {
            TextHit::Outside(_) => Some(true),
            _ => None,
        }
    }

    pub const fn is_inside(&self) -> bool {
        matches!(self, TextHit::Inside(_))
    }
}

pub trait TextLayout {
    /// Calculates the bounds of the text section.
    fn bounds(&self, section: &TextSection) -> Option<Rect>;

    /// Calculates the hit position of the text section.
    fn hit(&self, section: &TextSection, postition: Vec2) -> Option<TextHit>;
}
