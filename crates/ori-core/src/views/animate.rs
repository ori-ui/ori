use crate::{
    canvas::Canvas,
    event::{AnimationFrame, Event},
    layout::{Size, Space},
    theme::Theme,
    transition::Transition,
    view::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View},
};

/// Create a new [`Animate`].
pub fn animate<T, V, S>(
    animate: impl FnMut(&mut S, &mut EventCx, &mut T, &Event) -> Option<V> + 'static,
) -> Animate<T, V, S> {
    Animate::new(animate)
}

/// Animate a view when hot changes.
pub fn transition_hot<T, V>(
    transition: Transition,
    mut view: impl FnMut(&mut EventCx, &mut T, f32) -> V + 'static,
) -> Animate<T, V, f32> {
    let mut built = false;
    let mut was_hot = false;

    animate(move |t: &mut f32, cx, data: &mut T, event| {
        if cx.has_hot() != was_hot {
            was_hot = cx.has_hot();
            cx.request_animation_frame();
        }

        if let Some(AnimationFrame(dt)) = event.get() {
            if transition.step(t, cx.has_hot(), *dt) {
                cx.request_animation_frame();
                return Some(view(cx, data, transition.get(*t)));
            }
        }

        if !built {
            built = true;
            Some(view(cx, data, transition.get(*t)))
        } else {
            None
        }
    })
}

/// Animate a view when active changes.
pub fn transition_active<T, V>(
    transition: Transition,
    mut view: impl FnMut(&mut EventCx, &mut T, f32) -> V + 'static,
) -> Animate<T, V, f32> {
    let mut built = false;
    let mut was_active = false;

    animate(move |t: &mut f32, cx, data: &mut T, event| {
        if cx.has_active() != was_active {
            was_active = cx.has_active();
            cx.request_animation_frame();
        }

        if let Some(AnimationFrame(dt)) = event.get() {
            if transition.step(t, cx.has_active(), *dt) {
                cx.request_animation_frame();
                return Some(view(cx, data, transition.get(*t)));
            }
        }

        if !built {
            built = true;
            Some(view(cx, data, transition.get(*t)))
        } else {
            None
        }
    })
}

/// Animate a view when focused changes.
pub fn transition_focused<T, V>(
    transition: Transition,
    mut view: impl FnMut(&mut EventCx, &mut T, f32) -> V + 'static,
) -> Animate<T, V, f32> {
    let mut built = false;
    let mut was_focused = false;

    animate(move |t: &mut f32, cx, data: &mut T, event| {
        if cx.has_focused() != was_focused {
            was_focused = cx.has_focused();
            cx.request_animation_frame();
        }

        if let Some(AnimationFrame(dt)) = event.get() {
            if transition.step(t, cx.has_focused(), *dt) {
                cx.request_animation_frame();
                return Some(view(cx, data, transition.get(*t)));
            }
        }

        if !built {
            built = true;
            Some(view(cx, data, transition.get(*t)))
        } else {
            None
        }
    })
}

/// Animate a transition.
pub fn transition<T, V>(
    transition: Transition,
    active: bool,
    mut view: impl FnMut(&mut EventCx, &mut T, f32) -> V + 'static,
) -> Animate<T, V, f32> {
    let mut built = false;

    animate(move |t: &mut f32, cx, data: &mut T, event| {
        if let Some(AnimationFrame(dt)) = event.get() {
            if transition.step(t, active, *dt) {
                cx.request_animation_frame();
                return Some(view(cx, data, transition.get(*t)));
            }
        }

        if !built {
            built = true;
            Some(view(cx, data, transition.get(*t)))
        } else {
            None
        }
    })
}

/// A view that animates.
///
/// For an example, see [`animate`](https://github.com/ChangeCaps/ori/blob/main/examples/animate.rs).
pub struct Animate<T, V, S = ()> {
    /// The animation callback.
    #[allow(clippy::type_complexity)]
    pub animate: Box<dyn FnMut(&mut S, &mut EventCx, &mut T, &Event) -> Option<V>>,
    /// The theme to apply when building the view.
    pub theme: Theme,
}

impl<T, V, S> Animate<T, V, S> {
    /// Create a new [`Animate`].
    pub fn new(
        animate: impl FnMut(&mut S, &mut EventCx, &mut T, &Event) -> Option<V> + 'static,
    ) -> Self {
        Self {
            animate: Box::new(animate),
            theme: Theme::snapshot(),
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
        if let Some((ref mut state, ref mut view)) = state.view {
            view.event(state, cx, data, event);
        }

        let new_view = Theme::with_global(&mut self.theme, || {
            (self.animate)(&mut state.animate_state, cx, data, event)
        });

        if let Some(mut new_view) = new_view {
            match state.view {
                Some((ref mut view_state, ref mut view)) => {
                    new_view.rebuild(view_state, &mut cx.rebuild_cx(), data, view);
                    *view = new_view;
                }
                None => {
                    let mut view_state = new_view.build(&mut cx.build_cx(), data);
                    new_view.event(&mut view_state, cx, data, event);
                    state.view = Some((view_state, new_view));
                }
            }
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
