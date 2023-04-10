use glam::Vec2;

use crate::{Color, Mesh, Rect, Vertex};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quad {
    pub rect: Rect,
    pub background: Color,
    pub border_radius: [f32; 4],
    pub border_width: f32,
    pub border_color: Color,
}

impl Default for Quad {
    fn default() -> Self {
        Self {
            rect: Rect::default(),
            background: Color::TRANSPARENT,
            border_radius: [0.0; 4],
            border_width: 5.0,
            border_color: Color::BLACK,
        }
    }
}

impl Quad {
    pub const fn new(rect: Rect, background: Color) -> Self {
        Self {
            rect,
            background,
            border_radius: [0.0; 4],
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }

    pub fn mesh(self) -> Mesh {
        let mut mesh = Mesh::new();

        mesh.vertices.push(Vertex {
            position: self.rect.top_left(),
            color: self.background,
            uv: Vec2::new(0.0, 0.0),
        });
        mesh.vertices.push(Vertex {
            position: self.rect.top_right(),
            color: self.background,
            uv: Vec2::new(1.0, 0.0),
        });
        mesh.vertices.push(Vertex {
            position: self.rect.bottom_right(),
            color: self.background,
            uv: Vec2::new(1.0, 1.0),
        });
        mesh.vertices.push(Vertex {
            position: self.rect.bottom_left(),
            color: self.background,
            uv: Vec2::new(0.0, 1.0),
        });

        mesh.indices.push(0);
        mesh.indices.push(1);
        mesh.indices.push(2);
        mesh.indices.push(0);
        mesh.indices.push(2);
        mesh.indices.push(3);

        mesh
    }
}
