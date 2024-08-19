use ori_macro::Build;

use crate::{
    canvas::{Curve, FillRule, Paint},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    rebuild::Rebuild,
    style::Styles,
    view::View,
};

/// Create a new [`Painter`] view.
pub fn painter<T>(draw: impl FnMut(&mut DrawCx, &mut T) + 'static) -> Painter<T> {
    Painter::new(draw)
}

/// Create a new [`Painter`] view that draws a circle.
pub fn circle<T>(radius: f32, paint: impl Into<Paint>) -> Painter<T> {
    Painter::new({
        let paint = paint.into();

        move |cx, _| {
            cx.fill(
                Curve::circle(cx.rect().center(), radius),
                FillRule::NonZero,
                paint.clone(),
            );
        }
    })
    .size(Size::all(radius * 2.0))
}

/// Create a new [`Painter`] view that draws an ellipse.
pub fn ellipse<T>(size: Size, paint: impl Into<Paint>) -> Painter<T> {
    Painter::new({
        let paint = paint.into();

        move |cx, _| {
            cx.fill(Curve::oval(cx.rect()), FillRule::NonZero, paint.clone());
        }
    })
    .size(size)
}

/// Create a new [`Painter`] view that draws a rectangle.
pub fn rect<T>(size: Size, paint: impl Into<Paint>) -> Painter<T> {
    Painter::new({
        let paint = paint.into();

        move |cx, _| {
            cx.fill(Curve::rect(cx.rect()), FillRule::NonZero, paint.clone());
        }
    })
    .size(size)
}

/// A view that draws something.
///
/// The painter takes up as much space as possible.
#[derive(Build, Rebuild)]
pub struct Painter<T> {
    /// The draw function.
    #[allow(clippy::type_complexity)]
    pub draw: Box<dyn FnMut(&mut DrawCx, &mut T)>,

    /// The size of the view.
    pub size: Option<Size>,
}

impl<T> Painter<T> {
    /// Create a new [`Painter`] view.
    pub fn new(mut draw: impl FnMut(&mut DrawCx, &mut T) + 'static) -> Self {
        let mut snapshot = Styles::snapshot();

        Self {
            draw: Box::new(move |cx, data| snapshot.as_context(|| draw(cx, data))),

            size: None,
        }
    }
}

impl<T> View<T> for Painter<T> {
    type State = ();

    fn build(&mut self, _cx: &mut BuildCx, _data: &mut T) -> Self::State {}

    fn rebuild(&mut self, _state: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);
    }

    fn event(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut EventCx,
        _data: &mut T,
        _event: &Event,
    ) {
    }

    fn layout(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        match self.size {
            Some(size) => space.fit(size),
            None => space.max,
        }
    }

    fn draw(&mut self, _state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        (self.draw)(cx, data);
    }
}
