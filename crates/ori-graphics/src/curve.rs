use std::f32::consts::PI;

use glam::Vec2;

use crate::{Color, Mesh, Vertex};

/// A curve.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Curve {
    /// The points of the curve.
    pub points: Vec<Vec2>,
}

impl Curve {
    /// Creates an empty curve.
    pub const fn new() -> Self {
        Self { points: vec![] }
    }

    /// Creates an arc.
    ///
    /// # Arguments
    /// - `center`: The center of the arc.
    /// - `radius`: The radius of the arc.
    /// - `start_angle`: The start angle of the arc.
    /// - `end_angle`: The end angle of the arc.
    pub fn arc_center_angle(center: Vec2, radius: f32, start_angle: f32, end_angle: f32) -> Self {
        let mut curve = Curve::new();

        let length = (end_angle - start_angle).abs() * radius;
        // calculate the step in radians for a distance of 1 pixel
        let step = 1.0 / length;

        if step <= f32::EPSILON {
            return curve;
        }

        let mut angle = start_angle;
        while angle < end_angle {
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();

            curve.add_point(Vec2::new(x, y));

            angle += step;
        }

        curve
    }

    /// Creates a parametric curve.
    ///
    /// # Arguments
    /// - `f`: The function that returns the point at a given time.
    /// - `start`: The start time.
    /// - `end`: The end time.
    pub fn parametric(f: impl Fn(f32) -> Vec2, start: f32, end: f32) -> Self {
        let mut curve = Curve::new();

        let mut t = start;
        while t < end {
            let point = f(t);
            curve.add_point(point);

            let epsilon = 0.0001;
            let gradient = (f(t + epsilon / 2.0) - f(t - epsilon / 2.0)) / epsilon;

            t += 1.0 / gradient.length();
        }

        curve
    }

    /// Returns the number of points in the curve.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Returns whether the curve is empty.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Adds a point to the curve.
    pub fn add_point(&mut self, point: Vec2) {
        self.points.push(point);
    }

    /// Removes a point from the curve at `index`.
    pub fn remove_point(&mut self, index: usize) {
        self.points.remove(index);
    }

    /// Clears the curve.
    pub fn clear(&mut self) {
        self.points.clear();
    }

    /// Creates a mesh with rounded ends.
    pub fn rounded_mesh(&self, thickness: f32, color: Color) -> Mesh {
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
            let point = center + Vec2::new(angle.cos(), angle.sin()) * thickness;
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

                let hat = Vec2::new(prev_center.y, -prev_center.x);

                let offset = hat * thickness;

                let vertex_a = Vertex::new_color(center + offset, color);
                let vertex_b = Vertex::new_color(center - offset, color);

                let i = mesh.vertices.len() as u32;
                mesh.vertices.push(vertex_a);
                mesh.vertices.push(vertex_b);

                // add indices for prev center
                mesh.indices.push(i - 2);
                mesh.indices.push(i - 1);
                mesh.indices.push(i + 0);
                mesh.indices.push(i - 1);
                mesh.indices.push(i + 1);
                mesh.indices.push(i + 0);
            } else if i > 0 {
                let prev = self.points[i - 1];
                let center = self.points[i];
                let next = self.points[i + 1];

                let prev_center = (center - prev).normalize();
                let center_next = (next - center).normalize();

                let hat_a = Vec2::new(prev_center.y, -prev_center.x);
                let hat_b = Vec2::new(center_next.y, -center_next.x);

                let offset = (hat_a + hat_b).normalize() * thickness;

                let vertex_a = Vertex::new_color(center + offset, color);
                let vertex_b = Vertex::new_color(center - offset, color);

                let i = mesh.vertices.len() as u32;
                mesh.vertices.push(vertex_a);
                mesh.vertices.push(vertex_b);

                // add indices for prev center
                mesh.indices.push(i - 2);
                mesh.indices.push(i - 1);
                mesh.indices.push(i + 0);
                mesh.indices.push(i - 1);
                mesh.indices.push(i + 1);
                mesh.indices.push(i + 0);
            } else {
                let center = self.points[i];
                let next = self.points[i + 1];

                let center_next = (next - center).normalize();

                let hat = Vec2::new(center_next.y, -center_next.x);

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
            let point = center + Vec2::new(angle.cos(), angle.sin()) * thickness;
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
    type Item = Vec2;
    type IntoIter = std::vec::IntoIter<Vec2>;

    fn into_iter(self) -> Self::IntoIter {
        self.points.into_iter()
    }
}

impl<'a> IntoIterator for &'a Curve {
    type Item = &'a Vec2;
    type IntoIter = std::slice::Iter<'a, Vec2>;

    fn into_iter(self) -> Self::IntoIter {
        self.points.iter()
    }
}

impl<'a> IntoIterator for &'a mut Curve {
    type Item = &'a mut Vec2;
    type IntoIter = std::slice::IterMut<'a, Vec2>;

    fn into_iter(self) -> Self::IntoIter {
        self.points.iter_mut()
    }
}
