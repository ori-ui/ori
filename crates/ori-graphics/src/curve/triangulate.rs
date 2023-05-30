use std::collections::BTreeSet;

use glam::Vec2;

use crate::{Color, Curve, Mesh, Vertex};

impl Curve {
    pub fn fill(&self, color: Color) -> Mesh {
        let indices = Triangulation::triangulate_curve(self);
        let mut vertices = Vec::with_capacity(self.len());

        for &point in &self.points {
            vertices.push(Vertex {
                position: point,
                uv: Vec2::ZERO,
                color,
            });
        }

        Mesh {
            vertices,
            indices,
            image: None,
        }
    }
}

#[derive(Default)]
struct Triangulation<'a> {
    points: &'a [Vec2],
    relations: Vec<[usize; 2]>,
    convex: BTreeSet<usize>,
    concave: BTreeSet<usize>,
    ears: BTreeSet<usize>,
}

impl<'a> Triangulation<'a> {
    fn triangulate_curve(curve: &'a Curve) -> Vec<u32> {
        let mut this = Self {
            points: &curve.points,
            ..Default::default()
        };

        for i in 0..curve.len() {
            let prev = (i + curve.points.len() - 1) % curve.points.len();
            let next = (i + 1) % curve.points.len();
            this.relations.push([prev, next]);
        }

        for i in 0..curve.len() {
            if this.is_convex(i) {
                this.convex.insert(i);
            } else {
                this.concave.insert(i);
            }
        }

        for i in 0..curve.len() {
            if this.is_ear(i) {
                this.ears.insert(i);
            }
        }

        let index_count = (curve.len() - 2) * 3;
        let mut indices = Vec::with_capacity(index_count);

        loop {
            let Some(&ear) = this.ears.first() else {
                panic!();
            };

            let [prev, next] = this.relations[ear];
            indices.push(prev as u32);
            indices.push(ear as u32);
            indices.push(next as u32);

            if indices.len() == index_count {
                break indices;
            }

            this.convex.remove(&ear);
            this.ears.remove(&ear);

            this.relations[prev][1] = next;
            this.relations[next][0] = prev;

            this.reconfigure(prev);
            this.reconfigure(next);
        }
    }

    fn cross(a: Vec2, b: Vec2) -> f32 {
        a.x * b.y - a.y * b.x
    }

    fn is_convex(&self, i: usize) -> bool {
        let [prev, next] = self.relations[i];
        let prev = self.points[prev];
        let next = self.points[next];
        let current = self.points[i];

        let a = prev - current;
        let b = next - current;

        Self::cross(a, b) > 0.0
    }

    fn is_ear(&self, i: usize) -> bool {
        let [prev_index, next_index] = self.relations[i];

        let p0 = self.points[prev_index];
        let p1 = self.points[i];
        let p2 = self.points[next_index];

        let v01 = p1 - p0;
        let v12 = p2 - p1;
        let v20 = p0 - p2;

        for &i in &self.concave {
            if i == prev_index || i == next_index {
                continue;
            }

            let p = self.points[i];

            let v0p = p - p0;
            let v1p = p - p1;
            let v2p = p - p2;

            let c0 = Self::cross(v01, v0p);
            let c1 = Self::cross(v12, v1p);
            let c2 = Self::cross(v20, v2p);

            if c0 <= 0.0 && c1 <= 0.0 && c2 <= 0.0 {
                return false;
            }
        }

        true
    }

    fn reconfigure(&mut self, i: usize) {
        if self.concave.contains(&i) {
            if self.is_convex(i) {
                self.concave.remove(&i);
                self.convex.insert(i);

                if self.is_ear(i) {
                    self.ears.insert(i);
                }
            }
        } else if self.is_ear(i) {
            self.ears.insert(i);
        } else {
            self.ears.remove(&i);
        }
    }
}
