use std::ops::Range;

use glam::Vec2;

use crate::{Context, StyleAttributeEnum};

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
            min: self.min,
            max: Vec2::splat(f32::INFINITY),
        }
    }

    pub fn loose_x(self) -> Self {
        Self {
            min: self.min,
            max: Vec2::new(f32::INFINITY, self.max.y),
        }
    }

    pub fn loose_y(self) -> Self {
        Self {
            min: self.min,
            max: Vec2::new(self.max.x, f32::INFINITY),
        }
    }

    pub fn shrink(self, size: Vec2) -> Self {
        Self {
            min: Vec2::max(self.min - size, Vec2::ZERO),
            max: Vec2::max(self.max - size, Vec2::ZERO),
        }
    }

    pub fn constrain(self, size: Vec2) -> Vec2 {
        size.clamp(self.min, self.max)
    }

    pub fn width(self) -> Range<f32> {
        self.min.x..self.max.x
    }

    pub fn height(self) -> Range<f32> {
        self.min.y..self.max.y
    }

    pub fn with_margin(self, margin: Margin) -> Self {
        Self {
            min: Vec2::max(self.min - margin.size(), Vec2::ZERO),
            max: Vec2::max(self.max - margin.size(), Vec2::ZERO),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Padding {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Padding {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    pub const fn new(left: f32, right: f32, top: f32, bottom: f32) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn from_style(context: &mut impl Context, bc: BoxConstraints) -> Self {
        let left = context.style_range_group("padding-left", "padding", 0.0..bc.max.x);
        let right = context.style_range_group("padding-right", "padding", 0.0..bc.max.x);
        let top = context.style_range_group("padding-top", "padding", 0.0..bc.max.y);
        let bottom = context.style_range_group("padding-bottom", "padding", 0.0..bc.max.y);

        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn top_left(self) -> Vec2 {
        Vec2::new(self.left, self.top)
    }

    pub fn size(self) -> Vec2 {
        Vec2::new(self.left + self.right, self.top + self.bottom)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Margin {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Margin {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    pub const fn new(left: f32, right: f32, top: f32, bottom: f32) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn from_style(context: &mut impl Context, bc: BoxConstraints) -> Self {
        let left = context.style_range_group("margin-left", "margin", 0.0..bc.max.x);
        let right = context.style_range_group("margin-right", "margin", 0.0..bc.max.x);
        let top = context.style_range_group("margin-top", "margin", 0.0..bc.max.y);
        let bottom = context.style_range_group("margin-bottom", "margin", 0.0..bc.max.y);

        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn top_left(self) -> Vec2 {
        Vec2::new(self.left, self.top)
    }

    pub fn size(self) -> Vec2 {
        Vec2::new(self.left + self.right, self.top + self.bottom)
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

    fn to_str(&self) -> &str {
        match self {
            Axis::Horizontal => "horizontal",
            Axis::Vertical => "vertical",
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

    fn to_str(&self) -> &str {
        match self {
            JustifyContent::Start => "start",
            JustifyContent::Center => "center",
            JustifyContent::End => "end",
            JustifyContent::SpaceBetween => "space-between",
            JustifyContent::SpaceAround => "space-around",
            JustifyContent::SpaceEvenly => "space-evenly",
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

    fn to_str(&self) -> &str {
        match self {
            AlignItems::Start => "start",
            AlignItems::Center => "center",
            AlignItems::End => "end",
            AlignItems::Stretch => "stretch",
        }
    }
}
