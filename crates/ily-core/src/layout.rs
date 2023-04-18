use std::ops::Range;

use glam::Vec2;

use crate::{StyleAttributeEnum, StyleAttributeValue};

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

    pub fn height(self) -> Range<f32> {
        self.min.y..self.max.y
    }

    pub fn width(self) -> Range<f32> {
        self.min.x..self.max.x
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

impl StyleAttributeEnum for Axis {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "horizontal" | "row" => Some(Axis::Horizontal),
            "vertical" | "column" => Some(Axis::Vertical),
            _ => None,
        }
    }
}

impl Into<StyleAttributeValue> for Axis {
    fn into(self) -> StyleAttributeValue {
        match self {
            Axis::Horizontal => StyleAttributeValue::String("horizontal".to_string()),
            Axis::Vertical => StyleAttributeValue::String("vertical".to_string()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JustifyContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl JustifyContent {
    pub fn justify(&self, children: &[f32], container_size: f32, gap: f32) -> Vec<f32> {
        if children.is_empty() {
            return Vec::new();
        }

        let mut positions = Vec::with_capacity(children.len());

        let total_gap = gap * (children.len() - 1) as f32;
        let total_size = children.iter().sum::<f32>() + total_gap;

        match self {
            JustifyContent::Start => {
                let mut position = 0.0;

                for &child in children {
                    positions.push(position);
                    position += child + gap;
                }
            }
            JustifyContent::Center => {
                let mut position = container_size / 2.0 - total_size / 2.0;

                for &child in children {
                    positions.push(position);
                    position += child + gap;
                }
            }
            JustifyContent::End => {
                let mut position = container_size - total_size;

                for &child in children {
                    positions.push(position);
                    position += child + gap;
                }
            }
            JustifyContent::SpaceBetween => {
                let gap = (container_size - total_size) / (children.len() - 1) as f32;

                let mut position = 0.0;

                for &child in children {
                    positions.push(position);
                    position += child + gap;
                }
            }
            JustifyContent::SpaceAround => {
                let gap = (container_size - total_size) / children.len() as f32;

                let mut position = gap / 2.0;

                for &child in children {
                    positions.push(position);
                    position += child + gap;
                }
            }
            JustifyContent::SpaceEvenly => {
                let gap = container_size / children.len() as f32;

                let mut position = gap / 2.0;

                for _ in children {
                    positions.push(position);
                    position += gap;
                }
            }
        }

        positions
    }
}

impl Default for JustifyContent {
    fn default() -> Self {
        Self::Start
    }
}

impl StyleAttributeEnum for JustifyContent {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "start" => Some(JustifyContent::Start),
            "center" => Some(JustifyContent::Center),
            "end" => Some(JustifyContent::End),
            "space-between" => Some(JustifyContent::SpaceBetween),
            "space-around" => Some(JustifyContent::SpaceAround),
            "space-evenly" => Some(JustifyContent::SpaceEvenly),
            _ => None,
        }
    }
}

impl Into<StyleAttributeValue> for JustifyContent {
    fn into(self) -> StyleAttributeValue {
        match self {
            JustifyContent::Start => StyleAttributeValue::String("start".to_string()),
            JustifyContent::Center => StyleAttributeValue::String("center".to_string()),
            JustifyContent::End => StyleAttributeValue::String("end".to_string()),
            JustifyContent::SpaceBetween => {
                StyleAttributeValue::String("space-between".to_string())
            }
            JustifyContent::SpaceAround => StyleAttributeValue::String("space-around".to_string()),
            JustifyContent::SpaceEvenly => StyleAttributeValue::String("space-evenly".to_string()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignItems {
    Start,
    Center,
    End,
    Stretch,
}

impl Default for AlignItems {
    fn default() -> Self {
        Self::Start
    }
}

impl AlignItems {
    pub fn align(&self, start: f32, end: f32, size: f32) -> f32 {
        match self {
            AlignItems::Start => start,
            AlignItems::Center => start + (end - start - size) / 2.0,
            AlignItems::End => end - size,
            AlignItems::Stretch => start,
        }
    }
}

impl StyleAttributeEnum for AlignItems {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "start" => Some(AlignItems::Start),
            "center" => Some(AlignItems::Center),
            "end" => Some(AlignItems::End),
            "stretch" => Some(AlignItems::Stretch),
            _ => None,
        }
    }
}

impl Into<StyleAttributeValue> for AlignItems {
    fn into(self) -> StyleAttributeValue {
        match self {
            AlignItems::Start => StyleAttributeValue::String("start".to_string()),
            AlignItems::Center => StyleAttributeValue::String("center".to_string()),
            AlignItems::End => StyleAttributeValue::String("end".to_string()),
            AlignItems::Stretch => StyleAttributeValue::String("stretch".to_string()),
        }
    }
}
