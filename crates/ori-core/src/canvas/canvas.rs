use crate::{
    layout::{Affine, Point, Rect, Size, Vector},
    view::ViewId,
};

use super::{Background, BorderRadius, BorderWidth, Color, Fragment, Primitive, Quad, Scene};

/// A canvas used for drawing a [`Scene`].
pub struct Canvas<'a> {
    scene: &'a mut Scene,
    /// The transform to apply to the canvas.
    pub transform: Affine,
    /// The depth of the canvas.
    pub depth: f32,
    /// The clip rectangle of the canvas.
    pub clip: Rect,
    /// The view that the canvas is being drawn for.
    pub view: Option<ViewId>,
}

impl<'a> Canvas<'a> {
    /// Create a new [`Canvas`].
    pub fn new(scene: &'a mut Scene, window_size: Size) -> Self {
        Self {
            scene,
            transform: Affine::IDENTITY,
            depth: 0.0,
            clip: Rect::min_size(Point::ZERO, window_size),
            view: None,
        }
    }

    /// Fork the canvas.
    ///
    /// Setting parameters on the forked canvas will not affect the original canvas.
    pub fn fork(&mut self) -> Canvas<'_> {
        Canvas {
            scene: self.scene,
            transform: self.transform,
            depth: self.depth,
            clip: self.clip,
            view: self.view,
        }
    }

    /// Create a new layer.
    pub fn layer(&mut self) -> Canvas<'_> {
        let mut canvas = self.fork();
        canvas.depth += 1.0;
        canvas
    }

    /// Translate the canvas.
    pub fn transform(&mut self, transform: Affine) {
        self.transform *= transform;
    }

    /// Translate the canvas.
    pub fn translate(&mut self, translation: Vector) {
        self.transform *= Affine::translate(translation);
    }

    /// Rotate the canvas.
    pub fn rotate(&mut self, angle: f32) {
        self.transform *= Affine::rotate(angle);
    }

    /// Scale the canvas.
    pub fn scale(&mut self, scale: Vector) {
        self.transform *= Affine::scale(scale);
    }

    /// Set the clip rectangle of the canvas.
    pub fn clip(&mut self, clip: Rect) {
        self.clip = self.clip.intersect(clip);
    }

    /// Run a function with a forked canvas.
    pub fn forked(&mut self, f: impl FnOnce(&mut Canvas<'_>)) {
        f(&mut self.fork());
    }

    /// Set the view that the canvas is being drawn for.
    ///
    /// This will enable hit testing for the view.
    pub fn set_view(&mut self, view: ViewId) {
        self.view = Some(view);
    }

    /// Temporarily set the view, see [`Canvas::set_view`].
    pub fn with_view<T>(&mut self, view: ViewId, f: impl FnOnce(&mut Self) -> T) -> T {
        let t = self.view;
        self.view = Some(view);
        let result = f(self);
        self.view = t;
        result
    }

    /// Draw a trigger to the canvas.
    ///
    /// This will enable hit testing without drawing anything.
    pub fn trigger(&mut self, view: ViewId, rect: Rect) {
        self.with_view(view, |canvas| {
            canvas.draw(Primitive::Trigger(rect));
        });
    }

    /// Draw a fragment to the canvas.
    ///
    /// This is the lowest-level drawing method, and will not apply any
    /// of `transform`, `depth`, or `clip`. These must be set manually.
    pub fn draw_fragment(&mut self, fragment: Fragment) {
        self.scene.push(fragment);
    }

    /// Draw a primitive to the canvas.
    pub fn draw(&mut self, primitive: impl Into<Primitive>) {
        let primitive = primitive.into();

        // only draw primitives that actually do something
        if primitive.is_ineffective() {
            return;
        }

        self.draw_fragment(Fragment {
            primitive,
            transform: self.transform,
            depth: self.depth,
            clip: self.clip,
            view: self.view,
            pixel_perfect: false,
        });
    }

    /// Draw a [`Primitive`] with pixel-perfect coordinates.
    pub fn draw_pixel_perfect(&mut self, primitive: impl Into<Primitive>) {
        self.draw_fragment(Fragment {
            primitive: primitive.into(),
            transform: self.transform.round(),
            depth: self.depth,
            clip: self.clip,
            view: self.view,
            pixel_perfect: true,
        });
    }

    /// Draw a quad to the canvas.
    pub fn draw_quad(
        &mut self,
        rect: Rect,
        background: impl Into<Background>,
        border_radius: impl Into<BorderRadius>,
        border_width: impl Into<BorderWidth>,
        border_color: impl Into<Color>,
    ) {
        let background = background.into();
        let border_radius = border_radius.into();
        let border_width = border_width.into();
        let border_color = border_color.into();

        self.draw_pixel_perfect(Quad {
            rect,
            background,
            border_radius,
            border_width,
            border_color,
        });
    }
}
