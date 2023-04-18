use bytemuck::{Pod, Zeroable};
use glam::Vec2;

use crate::{Color, ImageHandle, Rect};

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

    pub fn quad(rect: Rect) -> Self {
        let mut mesh = Mesh::new();

        mesh.vertices.push(Vertex {
            position: rect.top_left(),
            uv: Vec2::new(0.0, 0.0),
            ..Vertex::default()
        });
        mesh.vertices.push(Vertex {
            position: rect.top_right(),
            uv: Vec2::new(1.0, 0.0),
            ..Vertex::default()
        });
        mesh.vertices.push(Vertex {
            position: rect.bottom_right(),
            uv: Vec2::new(1.0, 1.0),
            ..Vertex::default()
        });
        mesh.vertices.push(Vertex {
            position: rect.bottom_left(),
            uv: Vec2::new(0.0, 1.0),
            ..Vertex::default()
        });

        mesh.indices.push(0);
        mesh.indices.push(1);
        mesh.indices.push(2);
        mesh.indices.push(2);
        mesh.indices.push(3);
        mesh.indices.push(0);

        mesh
    }

    pub fn image(rect: Rect, image: ImageHandle) -> Self {
        Self {
            image: Some(image),
            ..Self::quad(rect)
        }
    }

    pub fn vertex_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }

    pub fn index_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.indices)
    }
}
