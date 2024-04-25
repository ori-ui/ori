use std::{
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

use crate::layout::{Point, Vector};

use super::{Color, Mesh};

/// A curve.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Curve {
    /// The points of the curve.
    pub points: Vec<Point>,
}

impl Curve {
    /// The resolution of the curve, measured in pixels per point.
    pub const RESOLUTION: f32 = 3.0;

    /// Create an empty curve.
    pub const fn new() -> Self {
        Self { points: vec![] }
    }

    /// Extend the curve with the points from `other`.
    pub fn extend(&mut self, other: impl IntoIterator<Item = Point>) {
        self.points.extend(other);
    }

    /// Create an arc.
    ///
    /// # Arguments
    /// - `center`: The center of the arc.
    /// - `radius`: The radius of the arc.
    /// - `start_angle`: The start angle of the arc.
    /// - `end_angle`: The end angle of the arc.
    pub fn arc_center_angle(center: Point, radius: f32, start_angle: f32, end_angle: f32) -> Self {
        let mut curve = Curve::new();

        let length = (end_angle - start_angle).abs() * radius;
        let step = Self::RESOLUTION / length;

        if step <= f32::EPSILON {
            return curve;
        }

        let mut angle = start_angle;
        while angle < end_angle {
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();

            curve.push(Point::new(x, y));

            angle += step;
        }

        curve.push(Point::new(
            center.x + radius * end_angle.cos(),
            center.y + radius * end_angle.sin(),
        ));

        curve
    }

    /// Create a parametric curve.
    ///
    /// # Arguments
    /// - `f`: The function that returns the point at a given time.
    /// - `start`: The start time.
    /// - `end`: The end time.
    pub fn parametric(f: impl Fn(f32) -> Point, start: f32, end: f32) -> Self {
        let mut curve = Curve::new();

        let mut t = start;
        while t < end {
            let point = f(t);
            curve.push(point);

            let epsilon = 0.0001;
            let half = epsilon / 2.0;
            let gradient = (f(t + half) - f(t - half)) / epsilon;

            t += Self::RESOLUTION / gradient.length();
        }

        curve.push(f(end));

        curve
    }

    /// Get the number of points in the curve.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Get whether the curve is empty.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Add a point to the curve.
    pub fn push(&mut self, point: Point) {
        self.points.push(point);
    }

    /// Remove a point from the curve at `index`.
    pub fn remove(&mut self, index: usize) {
        self.points.remove(index);
    }

    /// Clear the curve.
    pub fn clear(&mut self) {
        self.points.clear();
    }

    /// Get an iterator over the points of the curve.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = Point> + '_ {
        self.points.iter().copied()
    }

    /// Get a mutable iterator over the points of the curve.
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Point> {
        self.points.iter_mut()
    }

    /// Creates a mesh with rounded ends.
    pub fn stroke(&self, thickness: f32, color: Color) -> Mesh {
        if self.is_empty() {
            return Mesh::new();
        }

        if self.len() == 1 {
            return Mesh::circle(self.points[0], thickness / 2.0, color);
        }

        let mut mesh = Mesh::new();

        let radius = thickness / 2.0;
        let end = self.points.len() - 1;

        let dir = Vector::normalize(self[0] - self[1]);
        let (a, b) = self.round_cap(&mut mesh, self[0], dir, radius, color);

        let (a, b) = self.stroke_segments(&mut mesh, a, b, radius, color);

        let dir = Vector::normalize(self[end] - self[end - 1]);
        let (c, d) = self.round_cap(&mut mesh, self[end], dir, radius, color);

        mesh.indices.push(a as u32);
        mesh.indices.push(b as u32);
        mesh.indices.push(d as u32);

        mesh.indices.push(c as u32);
        mesh.indices.push(d as u32);
        mesh.indices.push(b as u32);

        mesh
    }
}

impl IntoIterator for Curve {
    type Item = Point;
    type IntoIter = std::vec::IntoIter<Point>;

    fn into_iter(self) -> Self::IntoIter {
        self.points.into_iter()
    }
}

impl<'a> IntoIterator for &'a Curve {
    type Item = &'a Point;
    type IntoIter = std::slice::Iter<'a, Point>;

    fn into_iter(self) -> Self::IntoIter {
        self.points.iter()
    }
}

impl<'a> IntoIterator for &'a mut Curve {
    type Item = &'a mut Point;
    type IntoIter = std::slice::IterMut<'a, Point>;

    fn into_iter(self) -> Self::IntoIter {
        self.points.iter_mut()
    }
}

impl<I: SliceIndex<[Point]>> Index<I> for Curve {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.points[index]
    }
}

impl<I: SliceIndex<[Point]>> IndexMut<I> for Curve {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.points[index]
    }
}
