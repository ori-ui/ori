use std::f32::consts::PI;

use glam::Vec2;

use crate::{Color, Mesh, Vertex};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Curve {
    pub points: Vec<Vec2>,
}

impl Curve {
    pub fn new() -> Self {
        Self { points: vec![] }
    }

    pub fn arc(center: Vec2, radius: f32, start_angle: f32, end_angle: f32) -> Self {
        let mut curve = Curve::new();

        let mut angle = start_angle;
        let step = (end_angle - start_angle) / 32.0;

        while angle < end_angle {
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();

            curve.add_point(Vec2::new(x, y));

            angle += step;
        }

        curve
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    pub fn add_point(&mut self, point: Vec2) {
        self.points.push(point);
    }

    pub fn remove_point(&mut self, index: usize) {
        self.points.remove(index);
    }

    pub fn clear(&mut self) {
        self.points.clear();
    }

    pub fn rounded_mesh(self, thickness: f32, color: Color) -> Mesh {
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
