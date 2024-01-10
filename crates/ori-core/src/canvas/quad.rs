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

    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    fn uv(rect: Rect, point: Point) -> Point {
        let size = rect.size();
        let point = point - rect.top_left();

        point.to_point() / size
    }

    fn corner_data(&self, index: usize) -> (f32, f32, f32) {
        let radi: [f32; 4] = self.border_radius.into();
        let widths: [f32; 4] = self.border_width.into();

        let radius = radi[index];
        let start_width = widths[index];
        let end_width = widths[(index + 3) % 4];

        (radius, start_width, end_width)
    }

    fn add_corner(
        &self,
        mesh: &mut Mesh,
        center_index: u32,
        corner: Point,
        angle: f32,
        index: usize,
    ) {
        let (radius, start_width, end_width) = self.corner_data(index);

        let length = FRAC_PI_2 * radius;
        let segments = u32::max(1, (length / Self::RESOLUTION).ceil() as u32);

        let sign = Vector::signum(corner - self.rect.center());
        let center = corner - sign * radius;

        for segment in 0..=segments {
            let fraction = segment as f32 / segments as f32;
            let angle = Vector::from_angle(angle + fraction * FRAC_PI_2);

            let width = Self::lerp(start_width, end_width, fraction);
            let inner_radius = radius - width;

            let inner_point = center + angle * inner_radius;

            let index = mesh.vertices.len() as u32;
            mesh.vertices.push(Vertex {
                position: inner_point,
                tex_coords: Self::uv(self.rect, inner_point),
                color: self.background.color,
            });

            if segment > 0 {
                mesh.indices.push(index - 1);
                mesh.indices.push(index);
                mesh.indices.push(center_index);
            }
        }
    }

    fn add_border_corner(&self, mesh: &mut Mesh, corner: Point, angle: f32, index: usize) -> bool {
        let (radius, start_width, end_width) = self.corner_data(index);

        if start_width == 0.0 && end_width == 0.0 {
            return false;
        }

        let length = FRAC_PI_2 * radius;
        let segments = u32::max(1, (length / Self::RESOLUTION).ceil() as u32);

        let sign = Vector::signum(corner - self.rect.center());
        let center = corner - sign * radius;

        for segment in 0..=segments {
            let fraction = segment as f32 / segments as f32;
            let angle = Vector::from_angle(angle + fraction * FRAC_PI_2);

            let width = Self::lerp(start_width, end_width, fraction);
            let inner_radius = radius - width;

            let inner_point = center + angle * inner_radius;
            let outer_point = center + angle * radius;

            let index = mesh.vertices.len() as u32;
            mesh.vertices.push(Vertex {
                position: inner_point,
                tex_coords: Self::uv(self.rect, inner_point),
                color: self.border_color,
            });
            mesh.vertices.push(Vertex {
                position: outer_point,
                tex_coords: Self::uv(self.rect, outer_point),
                color: self.border_color,
            });

            if segment > 0 {
                mesh.indices.push(index - 2);
                mesh.indices.push(index - 1);
                mesh.indices.push(index + 1);

                mesh.indices.push(index - 2);
                mesh.indices.push(index + 1);
                mesh.indices.push(index);
            }
        }

        true
    }

    fn connect_corners(mesh: &mut Mesh, center: u32, start: u32, end: u32) {
        mesh.indices.push(start);
        mesh.indices.push(end);
        mesh.indices.push(center);
    }

    fn connect_border(mesh: &mut Mesh, should_connect: bool, start: u32, end: u32) {
        if !should_connect {
            return;
        }

        mesh.indices.push(start);
        mesh.indices.push(start + 1);
        mesh.indices.push(end);

        mesh.indices.push(start + 1);
        mesh.indices.push(end + 1);
        mesh.indices.push(end);
    }

    /// Compute the mesh of the quad.
    pub fn compute_mesh(&self) -> Mesh {
        // TODO: this is jank to the max, but it's 4 AM and I'm tired

        let mut mesh = Mesh::new();
        mesh.texture = self.background.texture.clone();

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
        let tl_index = mesh.vertices.len() as u32;
        self.add_corner(&mut mesh, center_index, tl, PI, 0);
        let bl_index = mesh.vertices.len() as u32;
        self.add_corner(&mut mesh, center_index, bl, -FRAC_PI_2, 3);
        let br_index = mesh.vertices.len() as u32;
        self.add_corner(&mut mesh, center_index, br, 0.0, 2);
        let tr_index = mesh.vertices.len() as u32;
        self.add_corner(&mut mesh, center_index, tr, FRAC_PI_2, 1);
        let end_index = mesh.vertices.len() as u32 - 1;

        // connect the corner vertices
        Self::connect_corners(&mut mesh, center_index, end_index, tl_index);
        Self::connect_corners(&mut mesh, center_index, bl_index - 1, bl_index);
        Self::connect_corners(&mut mesh, center_index, br_index - 1, br_index);
        Self::connect_corners(&mut mesh, center_index, tr_index - 1, tr_index);

        // add the border vertices
        let tl_index = mesh.vertices.len() as u32;
        let tl_border = self.add_border_corner(&mut mesh, tl, PI, 0);
        let bl_index = mesh.vertices.len() as u32;
        let bl_border = self.add_border_corner(&mut mesh, bl, -FRAC_PI_2, 3);
        let br_index = mesh.vertices.len() as u32;
        let br_border = self.add_border_corner(&mut mesh, br, 0.0, 2);
        let tr_index = mesh.vertices.len() as u32;
        let tr_border = self.add_border_corner(&mut mesh, tr, FRAC_PI_2, 1);
        let end_index = mesh.vertices.len() as u32 - 2;

        // connect the border vertices
        Self::connect_border(&mut mesh, tr_border && tl_border, end_index, tl_index);
        Self::connect_border(&mut mesh, tl_border && bl_border, bl_index - 2, bl_index);
        Self::connect_border(&mut mesh, bl_border && br_border, br_index - 2, br_index);
        Self::connect_border(&mut mesh, br_border && tr_border, tr_index - 2, tr_index);

        mesh
    }
}
