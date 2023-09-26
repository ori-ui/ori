use crate::{
    canvas::Canvas,
    event::Event,
    layout::{Size, Space},
    theme::{theme_snapshot, Theme},
    view::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View},
};

/// Create a new [`Animate`].
pub fn animate<T, V, S>(
    animate: impl FnMut(&mut S, &mut EventCx, &mut T, &Event) -> V + 'static,
) -> Animate<T, V, S> {
    Animate::new(animate)
}

/// A view that animates.
///
/// For an example, see [`animate`](https://github.com/ChangeCaps/ori/blob/main/examples/animate.rs).
pub struct Animate<T, V, S = ()> {
    /// The animation callback.
    #[allow(clippy::type_complexity)]
    pub animate: Box<dyn FnMut(&mut S, &mut EventCx, &mut T, &Event) -> V>,
    /// The theme to apply when building the view.
    pub theme: Theme,
}

impl<T, V, S> Animate<T, V, S> {
    /// Create a new [`Animate`].
    pub fn new(animate: impl FnMut(&mut S, &mut EventCx, &mut T, &Event) -> V + 'static) -> Self {
        Self {
            animate: Box::new(animate),
            theme: theme_snapshot(),
        }
    }
}

#[doc(hidden)]
pub struct AnimateState<V: View<T>, S, T> {
    animate_state: S,
    view: Option<(V::State, V)>,
}

impl<T, V: View<T>, S: Default> View<T> for Animate<T, V, S> {
    type State = AnimateState<V, S, T>;

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        cx.request_animation_frame();

        AnimateState {
            animate_state: S::default(),
            view: None,
        }
    }

    fn rebuild(
        &mut self,
        _state: &mut Self::State,
        cx: &mut RebuildCx,
        _data: &mut T,
        _old: &Self,
    ) {
        cx.request_animation_frame();
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        let mut new_view = Theme::with_global(&mut self.theme, || {
            (self.animate)(&mut state.animate_state, cx, data, event)
        });

        if let Some((ref mut view_state, ref mut old_view)) = state.view {
            new_view.rebuild(view_state, &mut cx.rebuild_cx(), data, old_view);
            new_view.event(view_state, cx, data, event);

            *old_view = new_view;
        } else {
            let mut new_state = new_view.build(&mut cx.build_cx(), data);

            new_view.event(&mut new_state, cx, data, event);
            state.view = Some((new_state, new_view));
        }
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        if let Some((ref mut view_state, ref mut view)) = state.view {
            view.layout(view_state, cx, data, space)
        } else {
            space.min
        }
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        if let Some((ref mut view_state, ref mut view)) = state.view {
            view.draw(view_state, cx, data, canvas);
        }
    }
}
