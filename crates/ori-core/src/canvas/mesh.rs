use std::{f32::consts::PI, mem, slice};

use glam::Vec2;

use crate::{Color, Curve, Image};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vertex {
    pub position: Vec2,
    pub tex_coords: Vec2,
    pub color: Color,
}

impl Vertex {
    pub fn new_color(position: Vec2, color: Color) -> Self {
        Self {
            position,
            tex_coords: Vec2::ZERO,
            color,
        }
    }
}

/// A mesh of vertices and indices.
#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub image: Option<Image>,
}

impl Mesh {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a circle mesh with the given `center`, `radius`, and `color`.
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

    pub fn vertex_bytes(&self) -> &[u8] {
        let data = self.vertices.as_ptr() as *const u8;
        let len = self.vertices.len() * mem::size_of::<Vertex>();
        unsafe { slice::from_raw_parts(data, len) }
    }

    pub fn index_bytes(&self) -> &[u8] {
        let data = self.indices.as_ptr() as *const u8;
        let len = self.indices.len() * mem::size_of::<u32>();
        unsafe { slice::from_raw_parts(data, len) }
    }
}
