use std::f32::consts::PI;

use crate::{
    canvas::Vertex,
    image::{Image, Texture},
    layout::{Point, Rect, Vector},
};

use super::{BorderRadius, Color, Curve, Mesh};

/// A box shadow.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BoxShadow {
    /// The color of the shadow.
    pub color: Color,
    /// The blur radius of the shadow.
    pub blur: f32,
    /// The spread radius of the shadow.
    pub spread: f32,
    /// The offset of the shadow.
    pub offset: Vector,
}

impl From<f32> for BoxShadow {
    fn from(blur: f32) -> Self {
        Self {
            color: Color::BLACK,
            blur,
            ..Default::default()
        }
    }
}

impl From<(f32, f32)> for BoxShadow {
    fn from((blur, spread): (f32, f32)) -> Self {
        Self {
            color: Color::BLACK,
            blur,
            spread,
            ..Default::default()
        }
    }
}

impl From<(f32, f32, Vector)> for BoxShadow {
    fn from((blur, spread, offset): (f32, f32, Vector)) -> Self {
        Self {
            color: Color::BLACK,
            blur,
            spread,
            offset,
        }
    }
}

impl From<(f32, f32, Color)> for BoxShadow {
    fn from((blur, spread, color): (f32, f32, Color)) -> Self {
        Self {
            color,
            blur,
            spread,
            ..Default::default()
        }
    }
}

impl From<(f32, f32, Vector, Color)> for BoxShadow {
    fn from((blur, spread, offset, color): (f32, f32, Vector, Color)) -> Self {
        Self {
            color,
            blur,
            spread,
            offset,
        }
    }
}

impl BoxShadow {
    fn blur_image() -> Image {
        let size = 64;

        let mut pixels = vec![0; size * 4];

        let sigma = size as f32 / 4.2;
        let two_sigma_squared = 2.0 * sigma * sigma;
        let normalizer = 1.0 / (two_sigma_squared * PI).sqrt();

        for i in 0..size - 1 {
            let mut sum = 0.0;

            let half_size = size as i32 / 2;
            for j in -half_size..=half_size {
                let p = i as f32 + j as f32;

                let gauss = (-j as f32 * j as f32 / two_sigma_squared).exp();

                if p < half_size as f32 {
                    sum += gauss * normalizer;
                }
            }

            pixels[i * 4] = 255;
            pixels[i * 4 + 1] = 255;
            pixels[i * 4 + 2] = 255;
            pixels[i * 4 + 3] = (sum * 255.0) as u8;
        }

        Image::new(pixels, size as u32, 1)
    }

    fn add_corner(
        &self,
        mesh: &mut Mesh,
        point: Point,
        center_index: usize,
        radius: f32,
        angle: f32,
    ) {
        let inner_radius = radius;
        let outer_radius = radius + self.blur * 2.0;

        let length = (PI / 2.0 * outer_radius).abs();
        let steps = (length / Curve::RESOLUTION).round() as usize;

        for step in 0..=steps {
            let fraction = step as f32 / steps as f32;
            let angle = Vector::from_angle(angle + fraction * PI / 2.0);

            let inner_point = point + angle * inner_radius;
            let outer_point = point + angle * outer_radius;

            let index = mesh.vertices.len();

            mesh.vertices.push(Vertex {
                position: inner_point,
                tex_coords: Point::ZERO,
                color: self.color,
            });
            mesh.vertices.push(Vertex {
                position: outer_point,
                tex_coords: Point::X,
                color: self.color,
            });

            if step > 0 {
                mesh.indices.push(index as u32 - 2);
                mesh.indices.push(index as u32);
                mesh.indices.push(center_index as u32);

                mesh.indices.push(index as u32 - 2);
                mesh.indices.push(index as u32 - 1);
                mesh.indices.push(index as u32);

                mesh.indices.push(index as u32 - 1);
                mesh.indices.push(index as u32);
                mesh.indices.push(index as u32 + 1);
            }
        }
    }

    fn connect_corners(
        mesh: &mut Mesh,
        a_center: u32,
        a_inner: u32,
        a_outer: u32,
        b_center: u32,
        b_inner: u32,
        b_outer: u32,
    ) {
        mesh.indices.push(a_inner);
        mesh.indices.push(a_outer);
        mesh.indices.push(b_outer);

        mesh.indices.push(a_inner);
        mesh.indices.push(b_outer);
        mesh.indices.push(b_inner);

        mesh.indices.push(a_center);
        mesh.indices.push(b_center);
        mesh.indices.push(b_inner);

        mesh.indices.push(a_center);
        mesh.indices.push(b_inner);
        mesh.indices.push(a_inner);
    }

    /// Creates a mesh with rounded corners.
    pub fn mesh(&self, rect: Rect, radius: BorderRadius) -> Mesh {
        if self.color.a == 0.0 {
            return Mesh::new();
        }

        let spread_rect = rect.expand(self.spread - self.blur) + self.offset;

        let mut mesh = Mesh::new();

        mesh.texture = Some(Texture::Image(Self::blur_image()));

        let tl_radius = radius.top_left;
        let tr_radius = radius.top_right;
        let bl_radius = radius.bottom_left;
        let br_radius = radius.bottom_right;

        let tl = spread_rect.top_left() + Vector::new(tl_radius, tl_radius);
        let tr = spread_rect.top_right() + Vector::new(-tr_radius, tr_radius);
        let bl = spread_rect.bottom_left() + Vector::new(bl_radius, -bl_radius);
        let br = spread_rect.bottom_right() + Vector::new(-br_radius, -br_radius);

        mesh.vertices.push(Vertex::new_color(tl, self.color));
        mesh.vertices.push(Vertex::new_color(tr, self.color));
        mesh.vertices.push(Vertex::new_color(bl, self.color));
        mesh.vertices.push(Vertex::new_color(br, self.color));

        mesh.indices.push(0);
        mesh.indices.push(1);
        mesh.indices.push(2);

        mesh.indices.push(1);
        mesh.indices.push(2);
        mesh.indices.push(3);

        let tl_start_inner = mesh.vertices.len();
        let tl_start_outer = tl_start_inner + 1;
        self.add_corner(&mut mesh, tl, 0, tl_radius, PI);
        let tl_end_outer = mesh.vertices.len() - 1;
        let tl_end_inner = tl_end_outer - 1;

        let tr_start_inner = mesh.vertices.len();
        let tr_start_outer = tr_start_inner + 1;
        self.add_corner(&mut mesh, tr, 1, tr_radius, -PI / 2.0);
        let tr_end_outer = mesh.vertices.len() - 1;
        let tr_end_inner = tr_end_outer - 1;

        let br_start_inner = mesh.vertices.len();
        let br_start_outer = br_start_inner + 1;
        self.add_corner(&mut mesh, br, 3, br_radius, 0.0);
        let br_end_outer = mesh.vertices.len() - 1;
        let br_end_inner = br_end_outer - 1;

        let bl_start_inner = mesh.vertices.len();
        let bl_start_outer = bl_start_inner + 1;
        self.add_corner(&mut mesh, bl, 2, bl_radius, PI / 2.0);
        let bl_end_outer = mesh.vertices.len() - 1;
        let bl_end_inner = bl_end_outer - 1;

        Self::connect_corners(
            &mut mesh,
            0,
            tl_end_inner as u32,
            tl_end_outer as u32,
            1,
            tr_start_inner as u32,
            tr_start_outer as u32,
        );

        Self::connect_corners(
            &mut mesh,
            1,
            tr_end_inner as u32,
            tr_end_outer as u32,
            3,
            br_start_inner as u32,
            br_start_outer as u32,
        );

        Self::connect_corners(
            &mut mesh,
            3,
            br_end_inner as u32,
            br_end_outer as u32,
            2,
            bl_start_inner as u32,
            bl_start_outer as u32,
        );

        Self::connect_corners(
            &mut mesh,
            2,
            bl_end_inner as u32,
            bl_end_outer as u32,
            0,
            tl_start_inner as u32,
            tl_start_outer as u32,
        );

        mesh
    }
}
