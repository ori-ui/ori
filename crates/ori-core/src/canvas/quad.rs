use std::f32::consts::{FRAC_PI_2, PI};

use crate::layout::{Point, Rect, Vector};

use super::{Background, BorderRadius, BorderWidth, Color, Mesh, Vertex};

/// A quad primitive.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Quad {
    /// The rectangle of the quad.
    pub rect: Rect,
    /// The color of the quad.
    pub background: Background,
    /// The border radius of the quad.
    pub border_radius: BorderRadius,
    /// The border width of the quad.
    pub border_width: BorderWidth,
    /// The border color of the quad.
    pub border_color: Color,
}

impl Quad {
    /// Get whether the quad is ineffective, i.e. it has no effect on the canvas.
    pub fn is_ineffective(&self) -> bool {
        // if the rect has zero area, the quad is ineffective
        let area_zero = self.rect.area() == 0.0;

        let background_transparent = self.background.color.is_transparent();

        let border_zero = self.border_width == BorderWidth::ZERO;
        let border_transparent = self.border_color.is_transparent();

        let border_ineffective = border_zero || border_transparent;

        area_zero || (background_transparent && border_ineffective)
    }

    // number of segments per pixel
    const RESOLUTION: f32 = 3.0;

    fn add_corner(
        &self,
        mesh: &mut Mesh,
        center_index: u32,
        corner: Point,
        angle: f32,
        index: usize,
    ) -> CornerIndices {
        fn lerp(a: f32, b: f32, t: f32) -> f32 {
            a + (b - a) * t
        }

        fn uv(rect: Rect, point: Point) -> Point {
            let size = rect.size();
            let point = point - rect.top_left();

            point.to_point() / size
        }

        let radi: [f32; 4] = self.border_radius.into();
        let widths: [f32; 4] = self.border_width.into();

        let radius = radi[index];
        let start_width = widths[index];
        let end_width = widths[(index + 3) % 4];

        let length = FRAC_PI_2 * radius;
        let segments = u32::max(1, (length / Self::RESOLUTION).ceil() as u32);

        let sign = Vector::signum(corner - self.rect.center());
        let center = corner - sign * radius;

        let start = CornerIndices::start(mesh, start_width > 0.0);

        for segment in 0..=segments {
            let fraction = segment as f32 / segments as f32;
            let angle = Vector::from_angle(angle + fraction * FRAC_PI_2);

            let width = lerp(start_width, end_width, fraction);
            let inner_radius = radius - width;

            let inner_point = center + angle * inner_radius;
            let outer_point = center + angle * radius;

            let index = mesh.vertices.len() as u32;

            mesh.vertices.push(Vertex {
                position: inner_point,
                tex_coords: uv(self.rect, inner_point),
                color: self.background.color,
            });

            if segment > 0 {
                if width > 0.0 {
                    mesh.indices.push(index - 2);
                } else {
                    mesh.indices.push(index - 1);
                }

                mesh.indices.push(index);
                mesh.indices.push(center_index);
            }

            if width > 0.0 {
                mesh.vertices.push(Vertex {
                    position: outer_point,
                    tex_coords: uv(self.rect, outer_point),
                    color: self.border_color,
                });

                if segment > 0 {
                    mesh.indices.push(index - 2);
                    mesh.indices.push(index - 1);
                    mesh.indices.push(index);

                    mesh.indices.push(index - 1);
                    mesh.indices.push(index);
                    mesh.indices.push(index + 1);
                }
            }
        }

        let end = CornerIndices::end(mesh, end_width > 0.0);

        CornerIndices::new(start, end)
    }

    /// Compute the mesh of the quad.
    pub fn compute_mesh(&self) -> Mesh {
        let mut mesh = Mesh::new();

        // add the center vertex
        let center_index = mesh.vertices.len() as u32;
        mesh.vertices.push(Vertex {
            position: self.rect.center(),
            tex_coords: Point::ONE / 2.0,
            color: self.background.color,
        });

        let tl = self.rect.top_left();
        let tr = self.rect.top_right();
        let br = self.rect.bottom_right();
        let bl = self.rect.bottom_left();

        // add the corner vertices
        let tl = self.add_corner(&mut mesh, center_index, tl, PI, 0);
        let br = self.add_corner(&mut mesh, center_index, br, 0.0, 2);
        let bl = self.add_corner(&mut mesh, center_index, bl, -FRAC_PI_2, 3);
        let tr = self.add_corner(&mut mesh, center_index, tr, FRAC_PI_2, 1);

        // connect the corners
        tl.connect(&mut mesh, &bl, center_index);
        bl.connect(&mut mesh, &br, center_index);
        br.connect(&mut mesh, &tr, center_index);
        tr.connect(&mut mesh, &tl, center_index);

        //println!("self: {:?}, vertices: {}", self.rect, mesh.vertices.len());

        mesh
    }
}

struct CornerIndices {
    start_inner: u32,
    start_outer: Option<u32>,
    end_inner: u32,
    end_outer: Option<u32>,
}

impl CornerIndices {
    fn new(
        (start_inner, start_outer): (u32, Option<u32>),
        (end_inner, end_outer): (u32, Option<u32>),
    ) -> Self {
        Self {
            start_inner,
            start_outer,
            end_inner,
            end_outer,
        }
    }

    fn start(mesh: &Mesh, has_outer: bool) -> (u32, Option<u32>) {
        if has_outer {
            let start_inner = mesh.vertices.len() as u32;
            let start_outer = start_inner + 1;

            (start_inner, Some(start_outer))
        } else {
            let start_inner = mesh.vertices.len() as u32;
            let start_outer = None;

            (start_inner, start_outer)
        }
    }

    fn end(mesh: &Mesh, has_outer: bool) -> (u32, Option<u32>) {
        if has_outer {
            let end_inner = mesh.vertices.len() as u32 - 2;
            let end_outer = end_inner + 1;

            (end_inner, Some(end_outer))
        } else {
            let end_inner = mesh.vertices.len() as u32 - 1;
            let end_outer = None;

            (end_inner, end_outer)
        }
    }

    fn connect(&self, mesh: &mut Mesh, end: &Self, center: u32) {
        if let (Some(start_outer), Some(end_outer)) = (self.end_outer, end.start_outer) {
            mesh.indices.push(self.end_inner);
            mesh.indices.push(start_outer);
            mesh.indices.push(end_outer);

            mesh.indices.push(self.end_inner);
            mesh.indices.push(end_outer);
            mesh.indices.push(end.start_inner);
        }

        mesh.indices.push(center);
        mesh.indices.push(self.end_inner);
        mesh.indices.push(end.start_inner);
    }
}
