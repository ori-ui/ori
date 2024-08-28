use std::ops::{Deref, DerefMut};

use crate::{
    canvas::{BorderRadius, BorderWidth, Canvas, Curve},
    canvas::{FillRule, Mask, Paint, Stroke},
    layout::{Affine, Point, Rect, Size, Vector},
    text::{Fonts, TextBuffer},
    view::ViewState,
};

use super::BaseCx;

/// A context for drawing the view tree.
pub struct DrawCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
    pub(crate) view_state: &'a mut ViewState,
    pub(crate) transform: Affine,
    pub(crate) canvas: &'a mut Canvas,
    pub(crate) visible: Rect,
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
    const EVERYTHING: Rect = Rect::new(Point::all(f32::NEG_INFINITY), Point::all(f32::INFINITY));

    /// Create a new draw context.
    pub fn new(
        base: &'a mut BaseCx<'b>,
        view_state: &'a mut ViewState,
        canvas: &'a mut Canvas,
    ) -> Self {
        Self {
            base,
            view_state,
            transform: Affine::IDENTITY,
            canvas,
            visible: Self::EVERYTHING,
        }
    }

    /// Create a child context.
    pub fn child(&mut self) -> DrawCx<'_, 'b> {
        DrawCx {
            base: self.base,
            view_state: self.view_state,
            transform: self.transform,
            canvas: self.canvas,
            visible: self.visible,
        }
    }

    /// Check if a rect is visible.
    pub fn is_visible(&self, rect: Rect) -> bool {
        self.visible.intersects(rect)
    }

    /// Get the transform of the view.
    pub fn transform(&self) -> Affine {
        self.transform
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
        if !self.is_visible(rect) {
            return;
        }

        self.canvas.rect(rect, paint.into());
    }

    /// Draw a trigger rectangle.
    pub fn trigger(&mut self, rect: Rect) {
        if !self.is_visible(rect) {
            return;
        }

        self.canvas.trigger(rect, self.id());
    }

    /// Fill a curve.
    pub fn fill(&mut self, curve: Curve, fill: FillRule, paint: impl Into<Paint>) {
        if !self.is_visible(curve.bounds()) {
            return;
        }

        self.canvas.fill(curve, fill, paint.into());
    }

    /// Stroke a curve.
    pub fn stroke(&mut self, curve: Curve, stroke: impl Into<Stroke>, paint: impl Into<Paint>) {
        let stroke = stroke.into();

        if !self.is_visible(curve.bounds().inflate(stroke.width * 2.0)) {
            return;
        }

        self.canvas.stroke(curve, stroke, paint.into());
    }

    /// Draw a text buffer.
    pub fn text(&mut self, buffer: &TextBuffer, paint: impl Into<Paint>, offset: Vector) {
        self.text_raw(buffer.raw(), paint, offset);
    }

    /// Draw a raw text buffer.
    pub fn text_raw(
        &mut self,
        buffer: &cosmic_text::Buffer,
        paint: impl Into<Paint>,
        offset: Vector,
    ) {
        let scale = self.window().scale;
        let contexts = &mut *self.base.contexts;
        let canvas = &mut *self.canvas;

        contexts
            .get_or_default::<Fonts>()
            .draw_buffer(canvas, buffer, paint.into(), offset, scale);
    }

    /// Draw a rectangle with rounded corners and a border.
    pub fn quad(
        &mut self,
        rect: Rect,
        paint: impl Into<Paint>,
        border_radius: impl Into<BorderRadius>,
        border_width: impl Into<BorderWidth>,
        border_paint: impl Into<Paint>,
    ) {
        let radius = border_radius.into();
        let width = border_width.into();
        let rect = rect.round();

        let mut curve = Curve::new();
        curve.push_rect_with_radius(rect, radius);

        self.fill(curve, FillRule::NonZero, paint);

        let mut curve = Curve::new();
        curve.push_rect_with_borders(rect, radius, width);

        self.fill(curve, FillRule::NonZero, border_paint);
    }

    /// Draw an overlay, at `index`.
    pub fn overlay<T>(&mut self, index: i32, f: impl FnOnce(&mut DrawCx<'_, 'b>) -> T) -> T {
        self.canvas.overlay(index, |canvas| {
            let mut cx = DrawCx {
                base: self.base,
                view_state: self.view_state,
                transform: Affine::IDENTITY,
                canvas,
                visible: Self::EVERYTHING,
            };

            f(&mut cx)
        })
    }

    /// Draw a hoverable layer.
    pub fn hoverable(&mut self, f: impl FnOnce(&mut DrawCx<'_, 'b>)) {
        self.canvas.hoverable(self.id(), |canvas| {
            let mut cx = DrawCx {
                base: self.base,
                view_state: self.view_state,
                transform: self.transform,
                canvas,
                visible: Self::EVERYTHING,
            };

            f(&mut cx);
        });
    }

    /// Draw a layer that does not affect the canvas.
    pub fn void(&mut self, f: impl FnOnce(&mut DrawCx<'_, 'b>)) {
        self.canvas.void(|canvas| {
            let mut cx = DrawCx {
                base: self.base,
                view_state: self.view_state,
                transform: self.transform,
                canvas,
                visible: Rect::ZERO,
            };

            f(&mut cx);
        });
    }

    /// Draw a layer.
    pub fn layer(&mut self, transform: Affine, f: impl FnOnce(&mut DrawCx<'_, 'b>)) {
        let visible = self.visible.transform(transform.inverse());

        self.canvas.layer(transform, None, None, |canvas| {
            let mut cx = DrawCx {
                base: self.base,
                view_state: self.view_state,
                transform: self.transform * transform,
                canvas,
                visible,
            };

            f(&mut cx);
        });
    }

    /// Draw a layer with a translation.
    pub fn translate(&mut self, translation: Vector, f: impl FnOnce(&mut DrawCx<'_, 'b>)) {
        self.layer(Affine::translate(translation), f);
    }

    /// Draw a layer with a rotation.
    pub fn rotate(&mut self, angle: f32, f: impl FnOnce(&mut DrawCx<'_, 'b>)) {
        self.layer(Affine::rotate(angle), f);
    }

    /// Draw a layer with a mask.
    pub fn mask(&mut self, mask: impl Into<Mask>, f: impl FnOnce(&mut DrawCx<'_, 'b>)) {
        let mask = mask.into();
        let visible = self.visible.intersect(mask.curve.bounds());

        (self.canvas).layer(Affine::IDENTITY, Some(mask), None, |canvas| {
            let mut cx = DrawCx {
                base: self.base,
                view_state: self.view_state,
                transform: self.transform,
                canvas,
                visible,
            };

            f(&mut cx);
        });
    }
}
