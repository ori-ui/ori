use glam::Vec2;

use crate::{Affine, Color, Fragment, Primitive, Quad, Rect, Scene, Size};

pub struct Canvas<'a> {
    scene: &'a mut Scene,
    pub transform: Affine,
    pub depth: f32,
    pub clip: Rect,
}

impl<'a> Canvas<'a> {
    pub fn new(scene: &'a mut Scene, window_size: Size) -> Self {
        Self {
            scene,
            transform: Affine::IDENTITY,
            depth: 0.0,
            clip: Rect::min_size(Vec2::ZERO, window_size),
        }
    }

    pub fn layer(&mut self) -> Canvas<'_> {
        Canvas {
            scene: self.scene,
            transform: self.transform,
            depth: self.depth + 1.0,
            clip: self.clip,
        }
    }

    pub fn transform(&mut self, transform: Affine) {
        self.transform *= transform;
    }

    pub fn translate(&mut self, translation: Vec2) {
        self.transform *= Affine::translate(translation);
    }

    pub fn rotate(&mut self, angle: f32) {
        self.transform *= Affine::rotate(angle);
    }

    pub fn scale(&mut self, scale: Vec2) {
        self.transform *= Affine::scale(scale);
    }

    pub fn draw_fragment(&mut self, fragment: Fragment) {
        self.scene.push(fragment);
    }

    pub fn draw(&mut self, primitive: impl Into<Primitive>) {
        self.draw_fragment(Fragment {
            primitive: primitive.into(),
            transform: self.transform,
            depth: self.depth,
            clip: self.clip,
        });
    }

    pub fn draw_quad(
        &mut self,
        rect: Rect,
        color: Color,
        border_radius: [f32; 4],
        border_width: [f32; 4],
        border_color: Color,
    ) {
        self.draw(Quad {
            rect,
            color,
            border_radius,
            border_width,
            border_color,
        });
    }
}
