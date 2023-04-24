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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
    pub index: usize,
    pub rect: Rect,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Line {
    pub index: usize,
    pub glyphs: Vec<Glyph>,
    pub rect: Rect,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextHit {
    pub index: usize,
    pub inside: bool,
    pub delta: Vec2,
}
