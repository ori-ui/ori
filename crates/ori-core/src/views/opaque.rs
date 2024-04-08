use crate::{
    canvas::Canvas,
    event::Event,
    layout::{Size, Space},
    view::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View},
};

/// A view that bridges the gap between `impl View` and `impl View<T>`.
///
/// # Example
/// ```rust,no_run
/// # use ori_core::{view::View, views::*};
///
/// fn opaque_view() -> impl View {
///     button(text!("I am a button!"))
/// }
///
/// fn data_view<T>() -> impl View<T> {
///     opaque(opaque_view())
/// }
/// ```
pub fn opaque<V: View>(content: V) -> Opaque<V> {
    Opaque::new(content)
}

/// A view that bridges the gap between `impl View` and `impl View<T>`.
///
/// # Example
/// ```rust,no_run
/// # use ori_core::{view::View, views::*};
///
/// fn opaque_view() -> impl View {
///     button(text!("I am a button!"))
/// }
///
/// fn data_view<T>() -> impl View<T> {
///     opaque(opaque_view())
/// }
/// ```
pub struct Opaque<V> {
    /// The content view.
    pub content: V,
}

impl<V: View> Opaque<V> {
    /// Create a new opaque view.
    pub fn new(content: V) -> Self {
        Self { content }
    }
}

impl<T, V: View> View<T> for Opaque<V> {
    type State = V::State;

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        self.content.build(cx, &mut ())
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        self.content.rebuild(state, cx, &mut (), &old.content);
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, _data: &mut T, event: &Event) {
        self.content.event(state, cx, &mut (), event);
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        self.content.layout(state, cx, &mut (), space)
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        _data: &mut T,
        canvas: &mut Canvas,
    ) {
        self.content.draw(state, cx, &mut (), canvas);
    }
}
