use crate::{
    canvas::Canvas,
    event::{ActiveChanged, AnimationFrame, Event},
    layout::{Size, Space},
    theme::{theme_snapshot, Theme},
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
    mut view: impl FnMut(&mut EventCx, f32) -> V + 'static,
) -> Animate<T, V, f32> {
    let mut built = false;
    let mut was_hot = false;

    animate(move |t: &mut f32, cx, _data: &mut T, event| {
        if cx.has_hot() != was_hot {
            was_hot = cx.has_hot();
            cx.request_animation_frame();
        }

        if let Some(AnimationFrame(dt)) = event.get() {
            if transition.step(t, cx.is_hot(), *dt) {
                cx.request_animation_frame();
                return Some(view(cx, transition.get(*t)));
            }
        }

        if !built {
            built = true;
            Some(view(cx, transition.get(*t)))
        } else {
            None
        }
    })
}

/// Animate a view when active changes.
pub fn transition_active<T, V>(
    transition: Transition,
    mut view: impl FnMut(&mut EventCx, f32) -> V + 'static,
) -> Animate<T, V, f32> {
    let mut built = false;
    let mut was_active = false;

    animate(move |t: &mut f32, cx, _data: &mut T, event| {
        if cx.has_active() != was_active {
            was_active = cx.has_active();
            cx.request_animation_frame();
        }

        if let Some(AnimationFrame(dt)) = event.get() {
            if transition.step(t, cx.is_active(), *dt) {
                cx.request_animation_frame();
                return Some(view(cx, transition.get(*t)));
            }
        }

        if !built {
            built = true;
            Some(view(cx, transition.get(*t)))
        } else {
            None
        }
    })
}

/// Animate a view when focused changes.
pub fn transition_focused<T, V>(
    transition: Transition,
    mut view: impl FnMut(&mut EventCx, f32) -> V + 'static,
) -> Animate<T, V, f32> {
    let mut built = false;
    let mut was_focused = false;

    animate(move |t: &mut f32, cx, _data: &mut T, event| {
        if cx.is_focused() != was_focused {
            was_focused = cx.is_focused();
            cx.request_animation_frame();
        }

        if let Some(AnimationFrame(dt)) = event.get() {
            if transition.step(t, cx.is_focused(), *dt) {
                cx.request_animation_frame();
                return Some(view(cx, transition.get(*t)));
            }
        }

        if !built {
            built = true;
            Some(view(cx, transition.get(*t)))
        } else {
            None
        }
    })
}

/// Animate a transition.
pub fn transition<T, V>(
    transition: Transition,
    active: bool,
    mut view: impl FnMut(&mut EventCx, f32) -> V + 'static,
) -> Animate<T, V, f32> {
    let mut built = false;

    animate(move |t: &mut f32, cx, _data: &mut T, event| {
        if let Some(AnimationFrame(dt)) = event.get() {
            if transition.step(t, active, *dt) {
                cx.request_animation_frame();
                return Some(view(cx, transition.get(*t)));
            }
        }

        if !built {
            built = true;
            Some(view(cx, transition.get(*t)))
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
        fn update_view<T, V: View<T>>(
            cx: &mut EventCx,
            data: &mut T,
            state: &mut Option<(V::State, V)>,
            mut new_view: V,
            event: &Event,
        ) {
            if let Some((ref mut view_state, ref mut old_view)) = state {
                new_view.rebuild(view_state, &mut cx.rebuild_cx(), data, old_view);
                new_view.event(view_state, cx, data, event);

                *old_view = new_view;
            } else {
                let mut new_state = new_view.build(&mut cx.build_cx(), data);

                new_view.event(&mut new_state, cx, data, event);
                *state = Some((new_state, new_view));
            }
        }

        let new_view = Theme::with_global(&mut self.theme, || {
            (self.animate)(&mut state.animate_state, cx, data, event)
        });

        match new_view {
            Some(new_view) => update_view(cx, data, &mut state.view, new_view, event),
            None => {
                if let Some((ref mut view_state, ref mut view)) = state.view {
                    view.event(view_state, cx, data, event);
                }
            }
        };
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
