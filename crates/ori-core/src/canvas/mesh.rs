use std::{mem, slice};

use glam::Vec2;

use crate::{Color, Image};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vertex {
    pub position: Vec2,
    pub tex_coords: Vec2,
    pub color: Color,
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
