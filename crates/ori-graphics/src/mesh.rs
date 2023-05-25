use std::f32::consts::PI;

use bytemuck::{Pod, Zeroable};
use glam::Vec2;

use crate::{Color, ImageHandle, Quad, Rect};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Vertex {
    pub position: Vec2,
    pub uv: Vec2,
    pub color: Color,
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            uv: Vec2::ZERO,
            color: Color::WHITE,
        }
    }
}

impl Vertex {
    pub const fn new(position: Vec2) -> Self {
        Self {
            position,
            uv: Vec2::ZERO,
            color: Color::WHITE,
        }
    }

    pub const fn new_color(position: Vec2, color: Color) -> Self {
        Self {
            position,
            uv: Vec2::ZERO,
            color,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub image: Option<ImageHandle>,
}

impl Mesh {
    pub const fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            image: None,
        }
    }

    pub fn circle(center: Vec2, radius: f32, color: Color) -> Self {
        let mut mesh = Mesh::new();

        let center = Vertex::new_color(center, color);
        mesh.vertices.push(center);

        for i in 0..=60 {
            let angle = i as f32 / 60.0 * PI * 2.0;
            let x = angle.cos();
            let y = angle.sin();
            let vertex = Vertex::new_color(center.position + Vec2::new(x, y) * radius, color);
            mesh.vertices.push(vertex);

            if i < 60 {
                mesh.indices.push(0);
                mesh.indices.push(i as u32 + 1);
                mesh.indices.push(i as u32 + 2);
            }
        }

        mesh
    }

    pub fn quad(quad: Quad) -> Self {
        let inside_curve = quad.inside_curve();
        let outside_curve = quad.border_curve();

        let mut inside = inside_curve.fill(quad.background);
        let outside = outside_curve.stroke(quad.border_width / 2.0, quad.border_color);

        inside.extend(&outside);

        Self {
            vertices: inside.vertices,
            indices: inside.indices,
            image: None,
        }
    }

    pub fn extend(&mut self, other: &Self) {
        let offset = self.vertices.len() as u32;
        let new_indices = other.indices.iter().map(|i| i + offset).collect::<Vec<_>>();
        self.indices.extend_from_slice(new_indices.as_slice());
        self.vertices.extend_from_slice(&other.vertices);
    }

    pub fn image(rect: Rect, image: ImageHandle) -> Self {
        let mut mesh = Mesh::new();

        let tl = Vertex {
            position: rect.top_left(),
            uv: Vec2::ZERO,
            color: Color::WHITE,
        };
        let tr = Vertex {
            position: rect.top_right(),
            uv: Vec2::new(1.0, 0.0),
            color: Color::WHITE,
        };
        let br = Vertex {
            position: rect.bottom_right(),
            uv: Vec2::ONE,
            color: Color::WHITE,
        };
        let bl = Vertex {
            position: rect.bottom_left(),
            uv: Vec2::new(0.0, 1.0),
            color: Color::WHITE,
        };

        mesh.vertices.push(tl);
        mesh.vertices.push(tr);
        mesh.vertices.push(br);
        mesh.vertices.push(bl);

        mesh.indices.push(0);
        mesh.indices.push(1);
        mesh.indices.push(2);
        mesh.indices.push(0);
        mesh.indices.push(2);
        mesh.indices.push(3);

        mesh.image = Some(image);

        mesh
    }

    pub fn vertex_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }

    pub fn index_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.indices)
    }
}
