use std::{f32::consts::PI, mem, slice};

use crate::{
    image::Texture,
    layout::{Point, Vector},
};

use super::{Color, Curve};

/// A vertex in a [`Mesh`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vertex {
    /// The position of the vertex.
    pub position: Point,
    /// The texture coordinates of the vertex.
    pub tex_coords: Point,
    /// The color of the vertex.
    pub color: Color,
}

impl Vertex {
    /// Create a new vertex with `position` and color`.
    pub fn new_color(position: Point, color: Color) -> Self {
        Self {
            position,
            tex_coords: Point::ZERO,
            color,
        }
    }
}

/// A mesh containing vertices, indices and an optional image.
#[derive(Clone, Debug, Default)]
pub struct Mesh {
    /// The vertices of the mesh.
    pub vertices: Vec<Vertex>,
    /// The indices of the mesh.
    pub indices: Vec<u32>,
    /// The image of the mesh.
    pub texture: Option<Texture>,
}

impl Mesh {
    /// Create a new empty mesh.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a circle mesh with the given `center`, `radius`, and `color`.
    pub fn circle(center: Point, radius: f32, color: Color) -> Self {
        let mut mesh = Mesh::new();

        let center = Vertex::new_color(center, color);
        mesh.vertices.push(center);

        let circumference = radius * 2.0 * PI;
        let steps = (circumference / Curve::RESOLUTION).ceil() as usize;

        for i in 0..=steps {
            let angle = i as f32 / steps as f32 * PI * 2.0;
            let x = angle.cos();
            let y = angle.sin();
            let vertex = Vertex::new_color(center.position + Vector::new(x, y) * radius, color);
            mesh.vertices.push(vertex);

            if i < steps {
                mesh.indices.push(0);
                mesh.indices.push(i as u32 + 1);
                mesh.indices.push(i as u32 + 2);
            }
        }

        mesh
    }

    /// Set the image of the mesh.
    pub fn set_texture(&mut self, image: impl Into<Texture>) {
        self.texture = Some(image.into());
    }

    /// Get the bytes of the vertices.
    pub fn vertex_bytes(&self) -> &[u8] {
        let data = self.vertices.as_ptr() as *const u8;
        let len = self.vertices.len() * mem::size_of::<Vertex>();
        unsafe { slice::from_raw_parts(data, len) }
    }

    /// Get the bytes of the indices.
    pub fn index_bytes(&self) -> &[u8] {
        let data = self.indices.as_ptr() as *const u8;
        let len = self.indices.len() * mem::size_of::<u32>();
        unsafe { slice::from_raw_parts(data, len) }
    }

    /// Hit test the mesh.
    ///
    /// Returns true if any of the triangles in the mesh contains the given point.
    pub fn intersects_point(&self, point: Point) -> bool {
        // https://stackoverflow.com/a/2049593
        fn triangle_contains_point(a: Point, b: Point, c: Point, point: Point) -> bool {
            let v0 = c - a;
            let v1 = b - a;
            let v2 = point - a;

            let dot00 = v0.dot(v0);
            let dot01 = v0.dot(v1);
            let dot02 = v0.dot(v2);
            let dot11 = v1.dot(v1);
            let dot12 = v1.dot(v2);

            let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
            let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
            let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

            (u >= 0.0) && (v >= 0.0) && (u + v < 1.0)
        }

        for triangle in self.indices.chunks_exact(3) {
            let a = self.vertices[triangle[0] as usize].position;
            let b = self.vertices[triangle[1] as usize].position;
            let c = self.vertices[triangle[2] as usize].position;

            if triangle_contains_point(a, b, c, point) {
                return true;
            }
        }

        false
    }
}
