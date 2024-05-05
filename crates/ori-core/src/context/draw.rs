use std::ops::{Deref, DerefMut};

use crate::{
    canvas::{FillRule, Mask, Paint, Stroke},
    layout::{Affine, Point, Rect, Size, Vector},
    prelude::{BorderRadius, BorderWidth, Canvas, Curve, Image},
    text::{Fonts, TextBuffer},
    view::ViewState,
    window::Window,
};

use super::{BaseCx, RebuildCx};

/// A context for drawing the view tree.
pub struct DrawCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
    pub(crate) view_state: &'a mut ViewState,
    pub(crate) window: &'a mut Window,
    pub(crate) canvas: &'a mut Canvas,
}

impl<'a, 'b> Deref for DrawCx<'a, 'b> {
    type Target = BaseCx<'b>;

    fn deref(&self) -> &Self::Target {
        self.base
    }
}

impl<'a, 'b> DerefMut for DrawCx<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.base
    }
}

impl<'a, 'b> DrawCx<'a, 'b> {
    /// Create a new draw context.
    pub fn new(
        base: &'a mut BaseCx<'b>,
        view_state: &'a mut ViewState,
        window: &'a mut Window,
        canvas: &'a mut Canvas,
    ) -> Self {
        Self {
            base,
            view_state,
            window,
            canvas,
        }
    }

    /// Create a child context.
    pub fn child(&mut self) -> DrawCx<'_, 'b> {
        DrawCx {
            base: self.base,
            view_state: self.view_state,
            window: self.window,
            canvas: self.canvas,
        }
    }

    /// Get a rebuild context.
    pub fn rebuild_cx(&mut self) -> RebuildCx<'_, 'b> {
        RebuildCx::new(self.base, self.view_state, self.window)
    }

    /// Get the size of the view.
    pub fn size(&self) -> Size {
        self.view_state.size
    }

    /// Get the rect of the view in local space.
    pub fn rect(&self) -> Rect {
        Rect::min_size(Point::ZERO, self.size())
    }

    /// Get the canvas.
    pub fn canvas(&mut self) -> &mut Canvas {
        self.canvas
    }

    /// Draw a rectangle.
    pub fn fill_rect(&mut self, rect: Rect, paint: impl Into<Paint>) {
        self.canvas.rect(rect, paint.into());
    }

    /// Draw a trigger rectangle.
    pub fn trigger(&mut self, rect: Rect) {
        self.canvas.view(self.id(), |canvas| {
            canvas.trigger(rect);
        });
    }

    /// Fill a curve.
    pub fn fill_curve(&mut self, curve: Curve, fill: FillRule, paint: impl Into<Paint>) {
        self.canvas.fill(curve, fill, paint.into());
    }

    /// Stroke a curve.
    pub fn stroke(&mut self, curve: Curve, stroke: impl Into<Stroke>, paint: impl Into<Paint>) {
        self.canvas.stroke(curve, stroke.into(), paint.into());
    }

    /// Draw an image.
    pub fn image(&mut self, point: Point, image: Image) {
        self.canvas.image(point, image);
    }

    /// Draw a text buffer.
    pub fn text(&mut self, offset: Vector, buffer: &TextBuffer) {
        self.text_raw(offset, buffer.raw());
    }

    /// Draw a raw text buffer.
    pub fn text_raw(&mut self, offset: Vector, buffer: &cosmic_text::Buffer) {
        let contexts = &mut *self.base.contexts;
        let canvas = &mut *self.canvas;
        let scale = self.window.scale;

        contexts
            .get_or_default::<Fonts>()
            .draw_buffer(canvas, buffer, offset, scale);
    }

    /// Draw a rectangle with rounded corners and a border.
    pub fn quad(
        &mut self,
        rect: Rect,
        paint: impl Into<Paint>,
        border_radius: impl Into<BorderRadius>,
        _border_width: impl Into<BorderWidth>,
        _border_paint: impl Into<Paint>,
    ) {
        let mut curve = Curve::new();
        curve.push_rect_with_radius(rect, border_radius.into());

        self.fill_curve(curve, FillRule::NonZero, paint);
    }

    /// Draw a layer.
    pub fn layer(
        &mut self,
        transform: Affine,
        mask: Option<Mask>,
        f: impl FnOnce(&mut DrawCx<'_, 'b>),
    ) {
        self.canvas.layer(transform, mask, None, |canvas| {
            let mut cx = DrawCx {
                base: self.base,
                view_state: self.view_state,
                window: self.window,
                canvas,
            };

            f(&mut cx);
        });
    }

    /// Draw a hoverable layer.
    pub fn hoverable(&mut self, f: impl FnOnce(&mut DrawCx<'_, 'b>)) {
        self.canvas.view(self.id(), |canvas| {
            let mut cx = DrawCx {
                base: self.base,
                view_state: self.view_state,
                window: self.window,
                canvas,
            };

            f(&mut cx);
        });
    }

    /// Draw a layer with a transform.
    pub fn transform(&mut self, transform: Affine, f: impl FnOnce(&mut DrawCx<'_, 'b>)) {
        self.layer(transform, None, f);
    }

    /// Draw a layer with a translation.
    pub fn translate(&mut self, translation: Vector, f: impl FnOnce(&mut DrawCx<'_, 'b>)) {
        self.transform(Affine::translate(translation), f);
    }

    /// Draw a layer with a mask.
    pub fn mask(&mut self, mask: impl Into<Mask>, f: impl FnOnce(&mut DrawCx<'_, 'b>)) {
        self.layer(Affine::IDENTITY, Some(mask.into()), f);
    }
}
