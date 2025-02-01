use std::mem;

use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    style::Styles,
    view::View,
};

/// Create a view that applies a style to its content.
pub fn with_styles<V, T>(styles: impl Into<Styles>, content: V) -> WithStyles<V>
where
    V: View<T>,
{
    WithStyles::new(styles.into(), content)
}

/// A view that applies a style to its content.
pub struct WithStyles<V> {
    /// The content view.
    pub content: V,

    /// The style to apply.
    pub styles: Styles,
}

impl<V> WithStyles<V> {
    /// Create a new [`WithStyle`] view.
    pub fn new(styles: Styles, content: V) -> Self {
        Self { content, styles }
    }
}

#[doc(hidden)]
pub struct WithStyleState {
    base_version: u64,
    extra_version: u64,
    computed_styles: Styles,
}

impl<T, V: View<T>> View<T> for WithStyles<V> {
    type State = (WithStyleState, V::State);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let mut styles = cx.styles().clone();

        let base_version = styles.version();
        let extra_version = self.styles.version();

        styles.extend(mem::take(&mut self.styles));

        mem::swap(&mut styles, cx.context_mut());
        let content = self.content.build(cx, data);
        mem::swap(&mut styles, cx.context_mut());

        let state = WithStyleState {
            base_version,
            extra_version,
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
        if state.base_version != cx.styles().version()
            || state.extra_version != old.styles.version()
        {
            state.base_version = cx.styles().version();
            state.extra_version = old.styles.version();

            state.computed_styles = cx.styles().clone();
            state.computed_styles.extend(mem::take(&mut self.styles));
        }

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
    ) -> bool {
        mem::swap(&mut state.computed_styles, cx.context_mut());
        let handled = self.content.event(content, cx, data, event);
        mem::swap(&mut state.computed_styles, cx.context_mut());
        handled
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
