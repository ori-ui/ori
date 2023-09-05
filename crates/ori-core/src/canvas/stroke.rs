use std::f32::consts::PI;

use crate::layout::{Matrix, Point, Vector};

use super::{Color, Curve, Mesh, Vertex};

impl Curve {
    pub(super) fn round_cap(
        &self,
        mesh: &mut Mesh,
        center: Point,
        dir: Vector,
        radius: f32,
        color: Color,
    ) -> (usize, usize) {
        let length = radius * PI;
        let steps = (length / Self::RESOLUTION).ceil() as usize;

        // push the center of the cap
        let start = mesh.vertices.len() as u32;
        mesh.vertices.push(Vertex::new_color(center, color));

        let a = mesh.vertices.len() as u32;

        for i in 0..=steps {
            let angle = i as f32 / steps as f32 * PI;

            let matrix = Matrix::from_angle(angle);

            let offset = matrix * -dir.hat() * radius;
            let vertex = Vertex::new_color(center + offset, color);
            mesh.vertices.push(vertex);

            // push indices to make a triangle fan
            if i < steps {
                mesh.indices.push(start);
                mesh.indices.push(start + i as u32 + 1);
                mesh.indices.push(start + i as u32 + 2);
            }
        }

        (a as usize, mesh.vertices.len() - 1)
    }

    pub(super) fn stroke_segments(
        &self,
        mesh: &mut Mesh,
        mut a: usize,
        mut b: usize,
        radius: f32,
        color: Color,
    ) -> (usize, usize) {
        const MITER_LIMIT: f32 = 2.0;

        let end = self.points.len() - 1;

        for i in 1..end {
            let prev = self.points[i - 1];
            let curr = self.points[i];
            let next = self.points[i + 1];

            let da = Vector::normalize(curr - prev);
            let db = Vector::normalize(next - curr);

            let miter = da.hat() + db.hat();

            if miter.length() < MITER_LIMIT {
                let offset = miter * radius;

                // miter is within the limit, so we can use a miter join
                let c = mesh.vertices.len();
                mesh.vertices.push(Vertex::new_color(curr + offset, color));
                let d = mesh.vertices.len();
                mesh.vertices.push(Vertex::new_color(curr - offset, color));

                // create a quad with the vertices
                mesh.indices.push(a as u32);
                mesh.indices.push(b as u32);
                mesh.indices.push(d as u32);

                mesh.indices.push(c as u32);
                mesh.indices.push(d as u32);
                mesh.indices.push(a as u32);

                a = c;
                b = d;
            } else {
                // miter is outside the limit, so we can use a bevel join
                if da.cross(db) < 0.0 {
                    // the angle between the vectors is negative, so we need to flip the offset
                    let offset = da.hat() * radius;
                    let c = mesh.vertices.len();
                    mesh.vertices.push(Vertex::new_color(curr + offset, color));

                    let offset = miter * radius;
                    let d = mesh.vertices.len();
                    mesh.vertices.push(Vertex::new_color(curr - offset, color));

                    // create a quad with the vertices
                    mesh.indices.push(a as u32);
                    mesh.indices.push(b as u32);
                    mesh.indices.push(d as u32);

                    mesh.indices.push(c as u32);
                    mesh.indices.push(d as u32);
                    mesh.indices.push(a as u32);

                    a = c;
                    b = d;

                    let offset = db.hat() * radius;
                    let c = mesh.vertices.len();
                    mesh.vertices.push(Vertex::new_color(curr + offset, color));

                    mesh.indices.push(a as u32);
                    mesh.indices.push(b as u32);
                    mesh.indices.push(c as u32);

                    a = c;
                } else {
                    let offset = miter * radius;
                    let c = mesh.vertices.len();
                    mesh.vertices.push(Vertex::new_color(curr + offset, color));

                    let offset = da.hat() * radius;
                    let d = mesh.vertices.len();
                    mesh.vertices.push(Vertex::new_color(curr - offset, color));

                    // create a quad with the vertices
                    mesh.indices.push(a as u32);
                    mesh.indices.push(b as u32);
                    mesh.indices.push(d as u32);

                    mesh.indices.push(c as u32);
                    mesh.indices.push(d as u32);
                    mesh.indices.push(a as u32);

                    a = c;
                    b = d;

                    let offset = db.hat() * radius;
                    let c = mesh.vertices.len();
                    mesh.vertices.push(Vertex::new_color(curr - offset, color));

                    mesh.indices.push(a as u32);
                    mesh.indices.push(b as u32);
                    mesh.indices.push(c as u32);

                    b = c;
                }
            }
        }

        (a, b)
    }
}
