use std::ops::{Deref, DerefMut};

use crate::{
    canvas::Canvas,
    event::{ActiveChanged, Event, HotChanged, PointerEvent, SwitchFocus},
    layout::{Size, Space},
};

use super::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View, ViewState};

/// The state of a [`Pod`].
pub struct State<T, V: View<T> + ?Sized> {
    content: V::State,
    view_state: ViewState,
}

impl<T, V: View<T> + ?Sized> State<T, V> {
    /// Set the state to `active`.
    pub fn with_active(mut self, active: bool) -> Self {
        self.view_state.active = active;
        self
    }
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

/// A view that has separate [`ViewState`] from its content.
///
/// When calling for example [`View::event`], an [`EventCx`] is passed to the
/// function. This [`EventCx`] contains a mutable reference to a [`ViewState`] that is used to
/// keep track of state like whether the view is hot or active. If a pod is not used when
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
///     ...
/// }
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pod<V> {
    pub(crate) view: V,
}

impl<V> Pod<V> {
    /// Create a new content view.
    pub const fn new(view: V) -> Self {
        Self { view }
    }

    /// Build a content view.
    pub fn build<T>(cx: &mut BuildCx, f: impl FnOnce(&mut BuildCx) -> T) -> T {
        let mut new_cx = cx.child();
        f(&mut new_cx)
    }

    /// Rebuild a content view.
    pub fn rebuild(view_state: &mut ViewState, cx: &mut RebuildCx, f: impl FnOnce(&mut RebuildCx)) {
        view_state.prepare();

        let mut new_cx = cx.child();
        new_cx.view_state = view_state;

        f(&mut new_cx);

        new_cx.update();
        cx.view_state.propagate(view_state);
    }

    fn event_inner(
        view_state: &mut ViewState,
        cx: &mut EventCx,
        event: &Event,
        f: &mut impl FnMut(&mut EventCx, &Event),
    ) {
        view_state.prepare();

        let mut new_cx = cx.child();
        new_cx.transform *= view_state.transform;
        new_cx.view_state = view_state;

        f(&mut new_cx, event);
        new_cx.update();

        cx.view_state.propagate(view_state);
    }

    fn pointer_event(
        view_state: &mut ViewState,
        cx: &mut EventCx,
        event: &Event,
        pointer: &PointerEvent,
        f: &mut impl FnMut(&mut EventCx, &Event),
    ) {
        let transform = cx.transform * view_state.transform;
        let local = transform.inverse() * pointer.position;
        let hot = view_state.rect().contains(local) && !pointer.left && !event.is_handled();

        if view_state.is_hot() != hot && pointer.is_move() {
            view_state.set_hot(hot);
            Self::event_inner(view_state, cx, &Event::new(HotChanged(hot)), f);
        }

        Self::event_inner(view_state, cx, event, f);
    }

    /// Handle an event.
    pub fn event(
        view_state: &mut ViewState,
        cx: &mut EventCx,
        event: &Event,
        mut f: impl FnMut(&mut EventCx, &Event),
    ) {
        // we don't want `HotChanged` events to propagate
        if event.is::<HotChanged>() || event.is::<ActiveChanged>() {
            return;
        }

        if let Some(SwitchFocus::Next(focused)) | Some(SwitchFocus::Prev(focused)) = event.get() {
            if view_state.is_focused() {
                view_state.set_focused(false);
                focused.set(true);
            }
        }

        if let Some(pointer) = event.get::<PointerEvent>() {
            Self::pointer_event(view_state, cx, event, pointer, &mut f);
            return;
        }

        Self::event_inner(view_state, cx, event, &mut f);
    }

    /// Layout a content view.
    pub fn layout(
        view_state: &mut ViewState,
        cx: &mut LayoutCx,
        f: impl FnOnce(&mut LayoutCx) -> Size,
    ) -> Size {
        view_state.prepare_layout();

        let mut new_cx = cx.child();
        new_cx.view_state = view_state;

        let size = f(&mut new_cx);
        new_cx.update();

        view_state.size = size;
        cx.view_state.propagate(view_state);

        size
    }

    /// Draw a content view.
    pub fn draw(
        view_state: &mut ViewState,
        cx: &mut DrawCx,
        canvas: &mut Canvas,
        f: impl FnOnce(&mut DrawCx, &mut Canvas),
    ) {
        view_state.prepare_draw();

        // create the canvas layer
        let mut canvas = canvas.layer();
        canvas.transform *= view_state.transform;

        // create the draw context
        let mut new_cx = cx.layer();
        new_cx.view_state = view_state;

        // draw the content
        f(&mut new_cx, &mut canvas);
        new_cx.update();

        // propagate the view state
        cx.view_state.propagate(view_state);
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
        State {
            content: Self::build(cx, |cx| self.view.build(cx, data)),
            view_state: ViewState::default(),
        }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        Self::rebuild(&mut state.view_state, cx, |cx| {
            (self.view).rebuild(&mut state.content, cx, data, &old.view);
        });
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        Self::event(&mut state.view_state, cx, event, |cx, event| {
            (self.view).event(&mut state.content, cx, data, event);
        });
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        Self::layout(&mut state.view_state, cx, |cx| {
            (self.view).layout(&mut state.content, cx, data, space)
        })
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        Self::draw(&mut state.view_state, cx, canvas, |cx, canvas| {
            (self.view).draw(&mut state.content, cx, data, canvas);
        });
    }
}
