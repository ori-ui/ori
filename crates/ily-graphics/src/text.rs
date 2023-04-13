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
    pub position: Vec2,
    pub bounds: Vec2,
    pub scale: f32,
    pub h_align: TextAlign,
    pub v_align: TextAlign,
    pub text: String,
    pub font: Option<String>,
    pub color: Color,
}

impl Default for TextSection {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            bounds: Vec2::splat(f32::INFINITY),
            scale: 16.0,
            h_align: TextAlign::Start,
            v_align: TextAlign::Start,
            text: String::new(),
            font: None,
            color: Color::BLACK,
        }
    }
}

impl TextSection {
    pub fn set_rect(&mut self, rect: Rect) {
        self.position = Vec2::new(
            self.h_align.align(rect.min.x, rect.max.x),
            self.v_align.align(rect.min.y, rect.max.y),
        );
        self.bounds = rect.size();
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
