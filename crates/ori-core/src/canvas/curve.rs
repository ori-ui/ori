use std::{
    f32::consts::PI,
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

use crate::layout::{Point, Vector};

use super::{Color, Mesh, Vertex};

/// A curve.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Curve {
    /// The points of the curve.
    pub points: Vec<Point>,
}

impl Curve {
    /// The resolution of the curve, measured in pixels per point.
    pub const RESOLUTION: f32 = 5.0;

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

            curve.add_point(Point::new(x, y));

            angle += step;
        }

        curve.add_point(Point::new(
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
            curve.add_point(point);

            let epsilon = 0.0001;
            let half = epsilon / 2.0;
            let gradient = (f(t + half) - f(t - half)) / epsilon;

            t += Self::RESOLUTION / gradient.length();
        }

        curve.add_point(f(end));

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
    pub fn add_point(&mut self, point: Point) {
        self.points.push(point);
    }

    /// Remove a point from the curve at `index`.
    pub fn remove_point(&mut self, index: usize) {
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

    /// Returns true if the curve in counter-clockwise winding order, when interpreted as a polygon.
    ///
    /// This uses the shoelace formula, and runs in O(n) time.
    pub fn is_ccw(&self) -> bool {
        let mut sum = 0.0;
        for i in 0..self.len() {
            let a = self[i];
            let b = self[(i + 1) % self.len()];
            sum += (b.x - a.x) * (b.y + a.y);
        }
        sum > 0.0
    }

    /// Creates a mesh with rounded ends.
    pub fn stroke(&self, thickness: f32, color: Color) -> Mesh {
        if self.is_empty() {
            return Mesh::new();
        }

        if self.len() == 1 {
            return Mesh::circle(self.points[0], thickness, color);
        }

        let mut mesh = Mesh::new();

        // compute first cap
        let center = self.points[0];
        let next = self.points[1];
        let angle = (center - next).normalize();
        let angle = angle.y.atan2(angle.x);

        let index = mesh.vertices.len() as u32;
        mesh.vertices.push(Vertex::new_color(center, color));
        for i in -10..=10 {
            let angle = angle + i as f32 * PI / 20.0;
            let point = center + Vector::new(angle.cos(), angle.sin()) * thickness;
            mesh.vertices.push(Vertex::new_color(point, color));

            if i > -10 {
                let i = mesh.vertices.len() as u32;
                mesh.indices.push(index);
                mesh.indices.push(i - 2);
                mesh.indices.push(i - 1);
            }
        }

        // compute middle segments
        for i in 0..self.len() {
            if i == self.len() - 1 {
                let prev = self.points[i - 1];
                let center = self.points[i];

                let prev_center = (center - prev).normalize();
                let hat = prev_center.hat();

                let offset = hat * thickness;

                let vertex_a = Vertex::new_color(center + offset, color);
                let vertex_b = Vertex::new_color(center - offset, color);

                let i = mesh.vertices.len() as u32;
                mesh.vertices.push(vertex_a);
                mesh.vertices.push(vertex_b);

                // add indices for prev center
                mesh.indices.push(i - 2);
                mesh.indices.push(i - 1);
                mesh.indices.push(i);
                mesh.indices.push(i - 1);
                mesh.indices.push(i + 1);
                mesh.indices.push(i);
            } else if i > 0 {
                let prev = self.points[i - 1];
                let center = self.points[i];
                let next = self.points[i + 1];

                let a = (center - prev).normalize();
                let b = (next - center).normalize();
                let offset = (a.hat() + b.hat()).normalize();
                let angle = offset.angle_between(a.hat()) / 2.0;

                let offset = offset * thickness * (1.0 + angle.tan());

                let vertex_a = Vertex::new_color(center + offset, color);
                let vertex_b = Vertex::new_color(center - offset, color);

                let i = mesh.vertices.len() as u32;
                mesh.vertices.push(vertex_a);
                mesh.vertices.push(vertex_b);

                // add indices for prev center
                mesh.indices.push(i - 2);
                mesh.indices.push(i - 1);
                mesh.indices.push(i);
                mesh.indices.push(i - 1);
                mesh.indices.push(i + 1);
                mesh.indices.push(i);
            } else {
                let center = self.points[i];
                let next = self.points[i + 1];

                let center_next = (next - center).normalize();
                let hat = center_next.hat();

                let offset = hat * thickness;

                let vertex_a = Vertex::new_color(center + offset, color);
                let vertex_b = Vertex::new_color(center - offset, color);

                mesh.vertices.push(vertex_a);
                mesh.vertices.push(vertex_b);
            }
        }

        // compute last cap
        let center = self.points[self.len() - 1];
        let prev = self.points[self.len() - 2];
        let angle = (center - prev).normalize();
        let angle = angle.y.atan2(angle.x);

        let index = mesh.vertices.len() as u32;
        mesh.vertices.push(Vertex::new_color(center, color));
        for i in -10..=10 {
            let angle = angle + i as f32 * PI / 20.0;
            let point = center + Vector::new(angle.cos(), angle.sin()) * thickness;
            mesh.vertices.push(Vertex::new_color(point, color));

            if i > -10 {
                let i = mesh.vertices.len() as u32;
                mesh.indices.push(index);
                mesh.indices.push(i - 2);
                mesh.indices.push(i - 1);
            }
        }

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
