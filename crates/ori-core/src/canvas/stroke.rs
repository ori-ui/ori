use std::hash::{Hash, Hasher};

use crate::layout::Point;

use super::Curve;

/// Ways to draw the end of a line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LineCap {
    /// The end of the line is squared off.
    Butt,

    /// The end of the line is rounded.
    Round,

    /// The end of the line is squared off and extends past the end of the line.
    Square,
}

/// Ways to join two lines.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LineJoin {
    /// The lines are joined with a sharp corner.
    Miter,

    /// The lines are joined with a rounded corner.
    Round,

    /// The lines are joined with a beveled corner.
    Bevel,
}

/// Properties of a stroke.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stroke {
    /// The width of the stroke.
    pub width: f32,

    /// The miter limit of the stroke.
    pub miter: f32,

    /// The cap of the stroke.
    pub cap: LineCap,

    /// The join of the stroke.
    pub join: LineJoin,
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            width: 1.0,
            miter: 4.0,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
        }
    }
}

impl From<f32> for Stroke {
    fn from(value: f32) -> Self {
        Self {
            width: value,
            ..Default::default()
        }
    }
}

impl Hash for Stroke {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.width.to_bits().hash(state);
        self.miter.to_bits().hash(state);
        self.cap.hash(state);
        self.join.hash(state);
    }
}

impl Curve {
    fn offset_quad_bezier(&mut self, p0: Point, p1: Point, p2: Point, offset: f32) {}

    pub(super) fn stroke_impl(&self, stroke: Stroke) -> Curve {
        let mut curve = Curve::new();
        let mut prev = Point::ZERO;

        todo!()
    }
}
