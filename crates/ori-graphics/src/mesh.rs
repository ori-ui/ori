use std::f32::consts::PI;

use bytemuck::{Pod, Zeroable};
use glam::Vec2;

use crate::{Color, Curve, ImageHandle, Quad, Rect};

/// A vertex with position, UV coordinates, and color.
///
/// See [`Mesh`] for more information.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Vertex {
    /// The position of the vertex.
    pub position: Vec2,
    /// The UV coordinates of the vertex.
    pub uv: Vec2,
    /// The color of the vertex.
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
    /// Create a new vertex with the given `position`.
    pub const fn new(position: Vec2) -> Self {
        Self {
            position,
            uv: Vec2::ZERO,
            color: Color::WHITE,
        }
    }

    /// Create a new vertex with the given `position` and `color`.
    pub const fn new_color(position: Vec2, color: Color) -> Self {
        Self {
            position,
            uv: Vec2::ZERO,
            color,
        }
    }
}

/// A mesh of vertices and indices, and an optional image.
#[derive(Clone, Debug, Default)]
pub struct Mesh {
    /// The vertices of the mesh.
    pub vertices: Vec<Vertex>,
    /// The indices of the mesh.
    pub indices: Vec<u32>,
    /// The image of the mesh.
    pub image: Option<ImageHandle>,
}

impl Mesh {
    /// Create a new empty mesh.
    pub const fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            image: None,
        }
    }

    /// Creates a circle mesh with the given `center`, `radius`, and `color`.
    pub fn circle(center: Vec2, radius: f32, color: Color) -> Self {
        let mut mesh = Mesh::new();

        let center = Vertex::new_color(center, color);
        mesh.vertices.push(center);

        let circumference = radius * 2.0 * PI;
        let steps = (circumference / Curve::RESOLUTION).ceil() as usize;

        for i in 0..=steps {
            let angle = i as f32 / steps as f32 * PI * 2.0;
            let x = angle.cos();
            let y = angle.sin();
            let vertex = Vertex::new_color(center.position + Vec2::new(x, y) * radius, color);
            mesh.vertices.push(vertex);

            if i < steps {
                mesh.indices.push(0);
                mesh.indices.push(i as u32 + 1);
                mesh.indices.push(i as u32 + 2);
            }
        }

        mesh
    }

    /// Creates a mesh from a [`Quad`].
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

    /// Extends this mesh with the given `other` mesh.
    pub fn extend(&mut self, other: &Self) {
        let offset = self.vertices.len() as u32;
        let new_indices = other.indices.iter().map(|i| i + offset).collect::<Vec<_>>();
        self.indices.extend_from_slice(new_indices.as_slice());
        self.vertices.extend_from_slice(&other.vertices);
    }

    /// Creates a rectangle mesh with the given `rect` and `image`.
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

    /// Returns the bytes of the vertices.
    pub fn vertex_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }

    /// Returns the bytes of the indices.
    pub fn index_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.indices)
    }
}
