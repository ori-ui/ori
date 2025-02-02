use std::ops::{Deref, DerefMut};

use crate::{
    canvas::Canvas,
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::{Event, FocusTarget},
    layout::{Rect, Size, Space},
    style::{hash_style_key, Styles},
};

use super::{View, ViewState};

/// The state of a [`Pod`].
pub struct State<T, V: View<T> + ?Sized> {
    content: V::State,
    view_state: ViewState,
    prev_canvas: Canvas,
    prev_visible: Rect,
}

impl<T, V: View<T> + ?Sized> Deref for State<T, V> {
    type Target = ViewState;

    fn deref(&self) -> &Self::Target {
        &self.view_state
    }
}

impl<T, V: View<T> + ?Sized> DerefMut for State<T, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.view_state
    }
}

/// Create a new [`Pod`] view.
pub fn pod<V>(view: V) -> Pod<V> {
    Pod::new(view)
}

/// A view that has separate [`ViewState`] from its content.
///
/// When calling for example [`View::event`], an [`EventCx`] is passed to the
/// function. This [`EventCx`] contains a mutable reference to a [`ViewState`] that is used to
/// keep track of state like whether the view is hovered or active. If a pod is not used when
/// implementing a view, the [`View`] and the content share the same [`ViewState`]. This is
/// almost always an issue when the [`View`] wants to have a diffrent transform or size than
/// the content. See for example the [`Pad`](crate::views::Pad) view.
///
/// # Examples
/// ```ignore
/// use ori::prelude::*;
///
/// struct ContainerView<V> {
///     // We wrap the content in a Pod here
///     content: Pod<V>,
/// }
///
/// impl<V: View<T>, T> View<T> for ContainerView<V> {
///     // We use the Pod's state here
///     type State = State<T, V>;
///
///     fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
///         self.content.build(cx, data)
///     }
///
///     // ...
/// }
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pod<V> {
    pub(crate) view: V,
}

impl<V> Pod<V> {
    /// Create a new pod view.
    pub const fn new(view: V) -> Self {
        Self { view }
    }

    /// Call the [`View::event`] method on the content, only if the event hasn't been handled.
    pub fn event_maybe<T>(
        &mut self,
        handled: bool,
        state: &mut State<T, V>,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool
    where
        V: View<T>,
    {
        if !handled {
            return self.event(state, cx, data, event);
        }

        let _ = self.event(state, cx, data, &Event::Notify);
        true
    }

    /// Call a closure with the [`BuildCx`] provided by a pod.
    ///
    /// This will create both `T` and the new [`ViewState`].
    pub(crate) fn build_with<T>(
        cx: &mut BuildCx,
        f: impl FnOnce(&mut BuildCx) -> T,
    ) -> (T, ViewState) {
        let mut view_state = ViewState::default();

        if let Some(class) = cx.view_state.class() {
            let hash = hash_style_key(class.as_bytes());
            cx.context_mut::<Styles>().push_class_hash(hash);
        }

        let mut new_cx = cx.child();
        new_cx.view_state = &mut view_state;

        let state = f(&mut new_cx);

        cx.view_state.propagate(&mut view_state);

        if cx.view_state.class().is_some() {
            cx.context_mut::<Styles>().pop_class();
        }

        (state, view_state)
    }

    /// Call a closure with the [`RebuildCx`] provided by a pod.
    pub(crate) fn rebuild_with(
        view_state: &mut ViewState,
        cx: &mut RebuildCx,
        f: impl FnOnce(&mut RebuildCx),
    ) {
        view_state.prepare();

        if let Some(class) = cx.view_state.class() {
            let hash = hash_style_key(class.as_bytes());
            cx.context_mut::<Styles>().push_class_hash(hash);
        }

        let mut new_cx = cx.child();
        new_cx.view_state = view_state;

        f(&mut new_cx);

        cx.view_state.propagate(view_state);

        if cx.view_state.class().is_some() {
            cx.context_mut::<Styles>().pop_class();
        }
    }

    /// Call a closure with the [`EventCx`] provided by a pod.
    pub(crate) fn event_with(
        view_state: &mut ViewState,
        cx: &mut EventCx,
        event: &Event,
        f: impl FnMut(&mut EventCx, &Event) -> bool,
    ) -> bool {
        match event {
            Event::Animate(_) => {
                if !view_state.needs_animate() {
                    cx.view_state.propagate(view_state);

                    return false;
                }

                view_state.mark_animated();

                Self::event_with_inner(view_state, cx, event, f)
            }

            Event::FocusNext | Event::FocusPrev | Event::FocusWanted if view_state.is_focused() => {
                view_state.set_focused(false);

                Self::event_with_inner(view_state, cx, &Event::Notify, f);

                true
            }

            Event::FocusGiven(target) => {
                let focus_given = match target {
                    FocusTarget::Next | FocusTarget::Prev => view_state.is_focusable(),
                    FocusTarget::View(id) => view_state.id() == *id && view_state.is_focusable(),
                };

                if !focus_given {
                    return Self::event_with_inner(view_state, cx, event, f);
                }

                view_state.set_focused(true);

                Self::event_with_inner(view_state, cx, &Event::Notify, f);

                true
            }

            Event::WindowScaled(_) | Event::WindowResized(_) | Event::ForceLayout => {
                view_state.request_layout();

                Self::event_with_inner(view_state, cx, event, f)
            }

            event => Self::event_with_inner(view_state, cx, event, f),
        }
    }

    #[inline]
    fn event_with_inner(
        view_state: &mut ViewState,
        cx: &mut EventCx,
        event: &Event,
        f: impl FnOnce(&mut EventCx, &Event) -> bool,
    ) -> bool {
        view_state.set_hovered(cx.window().is_hovered(view_state.id()));
        view_state.prepare();

        if let Some(class) = cx.view_state.class() {
            let hash = hash_style_key(class.as_bytes());
            cx.context_mut::<Styles>().push_class_hash(hash);
        }

        let mut new_cx = cx.child();
        new_cx.transform *= view_state.transform;
        new_cx.view_state = view_state;

        let handled = f(&mut new_cx, event);

        view_state.prev_flags = view_state.flags;

        cx.view_state.propagate(view_state);

        if cx.view_state.class().is_some() {
            cx.context_mut::<Styles>().pop_class();
        }

        handled
    }

    /// Call a closure with the [`LayoutCx`] provided by a pod.
    pub(crate) fn layout_with(
        view_state: &mut ViewState,
        cx: &mut LayoutCx,
        f: impl FnOnce(&mut LayoutCx) -> Size,
    ) -> Size {
        view_state.mark_layed_out();

        if let Some(class) = cx.view_state.class() {
            let hash = hash_style_key(class.as_bytes());
            cx.context_mut::<Styles>().push_class_hash(hash);
        }

        let mut new_cx = cx.child();
        new_cx.view_state = view_state;

        view_state.size = f(&mut new_cx);

        if cx.view_state.class().is_some() {
            cx.context_mut::<Styles>().pop_class();
        }

        view_state.size
    }

    /// Call a closure with the [`DrawCx`] provided by a pod.
    pub(crate) fn draw_with(
        view_state: &mut ViewState,
        cx: &mut DrawCx,
        f: impl FnOnce(&mut DrawCx),
    ) {
        view_state.mark_drawn();

        if let Some(class) = cx.view_state.class() {
            let hash = hash_style_key(class.as_bytes());
            cx.context_mut::<Styles>().push_class_hash(hash);
        }

        // create the draw context
        let mut new_cx = cx.child();
        new_cx.view_state = view_state;

        // draw the content
        new_cx.transformed(new_cx.view_state.transform, |cx| {
            f(cx);
        });

        if cx.view_state.class().is_some() {
            cx.context_mut::<Styles>().pop_class();
        }
    }
}

impl<V> From<V> for Pod<V> {
    fn from(view: V) -> Self {
        Self::new(view)
    }
}

impl<V> Deref for Pod<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

impl<V> DerefMut for Pod<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.view
    }
}

impl<T, V: View<T>> View<T> for Pod<V> {
    type State = State<T, V>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let (content, view_state) = Self::build_with(cx, |cx| self.view.build(cx, data));

        State {
            content,
            view_state,
            prev_canvas: Canvas::new(),
            prev_visible: Rect::ZERO,
        }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        Self::rebuild_with(&mut state.view_state, cx, |cx| {
            (self.view).rebuild(&mut state.content, cx, data, &old.view);
        });
    }

    fn event(
        &mut self,
        state: &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        Self::event_with(&mut state.view_state, cx, event, |cx, event| {
            (self.view).event(&mut state.content, cx, data, event)
        })
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        Self::layout_with(&mut state.view_state, cx, |cx| {
            (self.view).layout(&mut state.content, cx, data, space)
        })
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        // we need to check if the view needs to be drawn here
        // since the flag gets cleared in draw function
        let needs_draw = state.view_state.needs_draw();

        Self::draw_with(&mut state.view_state, cx, |cx| {
            if !cx.is_visible(cx.rect()) {
                return;
            }

            // if the visible rect has changed since out last draw, we need to invalidate
            // the cached canvas, since content that previously wasn't visible might be now
            // and vice versa.
            //
            // this fixes a bug with the scroll view
            if needs_draw || state.prev_visible != cx.visible {
                // if the view needs to be drawn we draw it and save the canvas
                (self.view).draw(&mut state.content, cx, data);
                state.prev_canvas = cx.canvas.clone();
                state.prev_visible = cx.visible;
            } else {
                // if the view doesn't need to be drawn we just draw the saved canvas
                *cx.canvas = state.prev_canvas.clone();
            }
        });
    }
}
