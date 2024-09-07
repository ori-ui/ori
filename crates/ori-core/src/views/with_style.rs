use std::mem;

use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    style::Styles,
    view::View,
};

/// Create a view that applies a style to its content.
pub fn with_style<V: View<T>, T>(style: Styles, content: V) -> WithStyle<V> {
    WithStyle::new(content, style)
}

/// A view that applies a style to its content.
pub struct WithStyle<V> {
    /// The content view.
    pub content: V,

    /// The style to apply.
    pub style: Styles,
}

impl<V> WithStyle<V> {
    /// Create a new [`WithStyle`] view.
    pub fn new(content: V, style: Styles) -> Self {
        Self { content, style }
    }
}

#[doc(hidden)]
pub struct WithStyleState {
    computed_styles: Styles,
}

impl<T, V: View<T>> View<T> for WithStyle<V> {
    type State = (WithStyleState, V::State);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let mut styles = cx.styles().clone();

        styles.extend(mem::take(&mut self.style));

        mem::swap(&mut styles, cx.context_mut());
        let content = self.content.build(cx, data);
        mem::swap(&mut styles, cx.context_mut());

        let state = WithStyleState {
            computed_styles: styles,
        };

        (state, content)
    }

    fn rebuild(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        state.computed_styles = cx.styles().clone();
        state.computed_styles.extend(mem::take(&mut self.style));

        mem::swap(&mut state.computed_styles, cx.context_mut());
        self.content.rebuild(content, cx, data, &old.content);
        mem::swap(&mut state.computed_styles, cx.context_mut());
    }

    fn event(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) {
        mem::swap(&mut state.computed_styles, cx.context_mut());
        self.content.event(content, cx, data, event);
        mem::swap(&mut state.computed_styles, cx.context_mut());
    }

    fn layout(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        mem::swap(&mut state.computed_styles, cx.context_mut());
        let size = self.content.layout(content, cx, data, space);
        mem::swap(&mut state.computed_styles, cx.context_mut());
        size
    }

    fn draw(&mut self, (state, content): &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        mem::swap(&mut state.computed_styles, cx.context_mut());
        self.content.draw(content, cx, data);
        mem::swap(&mut state.computed_styles, cx.context_mut());
    }
}
