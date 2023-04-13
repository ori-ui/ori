use glam::Vec2;
use ily_graphics::TextAlign;

use crate::AttributeValue;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxConstraints {
    pub min: Vec2,
    pub max: Vec2,
}

impl BoxConstraints {
    pub const UNBOUNDED: Self = Self {
        min: Vec2::ZERO,
        max: Vec2::splat(f32::INFINITY),
    };

    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self {
            min: min.ceil(),
            max: max.ceil(),
        }
    }

    pub fn window(width: u32, height: u32) -> Self {
        Self {
            min: Vec2::ZERO,
            max: Vec2::new(width as f32, height as f32),
        }
    }

    pub fn loose(self) -> Self {
        Self {
            min: Vec2::ZERO,
            max: self.max,
        }
    }

    pub fn constrain(self, size: Vec2) -> Vec2 {
        size.clamp(self.min, self.max)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    #[default]
    Vertical,
}

impl Axis {
    pub const fn cross(self) -> Self {
        match self {
            Axis::Horizontal => Axis::Vertical,
            Axis::Vertical => Axis::Horizontal,
        }
    }

    pub const fn minor(self, size: Vec2) -> f32 {
        match self {
            Axis::Horizontal => size.y,
            Axis::Vertical => size.x,
        }
    }

    pub const fn major(self, size: Vec2) -> f32 {
        match self {
            Axis::Horizontal => size.x,
            Axis::Vertical => size.y,
        }
    }

    pub const fn pack(self, major: f32, minor: f32) -> Vec2 {
        match self {
            Axis::Horizontal => Vec2::new(major, minor),
            Axis::Vertical => Vec2::new(minor, major),
        }
    }
}

impl From<AttributeValue> for Option<Axis> {
    fn from(value: AttributeValue) -> Self {
        match value {
            AttributeValue::String(s) => match s.as_str() {
                "row" | "horizontal" => Some(Axis::Horizontal),
                "column" | "vertical" => Some(Axis::Vertical),
                _ => {
                    tracing::warn!("Invalid axis: {}", s);

                    None
                }
            },
            _ => None,
        }
    }
}

impl From<AttributeValue> for Option<TextAlign> {
    fn from(value: AttributeValue) -> Self {
        match value {
            AttributeValue::String(s) => match s.as_str() {
                "left" | "start" => Some(TextAlign::Start),
                "center" => Some(TextAlign::Center),
                "right" | "end" => Some(TextAlign::End),
                _ => {
                    tracing::warn!("Invalid text align: {}", s);

                    None
                }
            },
            _ => None,
        }
    }
}
