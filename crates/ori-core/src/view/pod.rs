use std::{
    ops::{Deref, DerefMut},
    time::Instant,
};

use crate::{
    canvas::Canvas,
    debug::{DebugDraw, DebugLayout, DebugTree},
    event::{Event, PointerLeft, PointerMoved, SwitchFocus},
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
        self.view_state.set_active(active);
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

/// Create a new [`Pod`] view.
pub fn pod<V>(view: V) -> Pod<V> {
    Pod::new(view)
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
    /// Create a new pod view.
    pub const fn new(view: V) -> Self {
        Self { view }
    }

    /// Build a pod view.
    pub fn build<T>(cx: &mut BuildCx, f: impl FnOnce(&mut BuildCx) -> T) -> (T, ViewState) {
        let mut view_state = ViewState::default();
        view_state.prepare();

        let mut new_cx = cx.child();
        new_cx.view_state = &mut view_state;

        let state = f(&mut new_cx);

        cx.view_state.propagate(&mut view_state);

        (state, view_state)
    }

    /// Rebuild a pod view.
    pub fn rebuild(view_state: &mut ViewState, cx: &mut RebuildCx, f: impl FnOnce(&mut RebuildCx)) {
        view_state.prepare();

        let mut new_cx = cx.child();
        new_cx.view_state = view_state;

        f(&mut new_cx);

        cx.view_state.propagate(view_state);
    }

    /// Handle an event.
    pub fn event(
        view_state: &mut ViewState,
        cx: &mut EventCx,
        event: &Event,
        mut f: impl FnMut(&mut EventCx, &Event),
    ) {
        if let Some(SwitchFocus::Next(focused)) | Some(SwitchFocus::Prev(focused)) = event.get() {
            if view_state.is_focused() {
                view_state.set_focused(false);
                focused.set(true);
            }
        }

        // update the hot state
        if event.is::<PointerMoved>() || event.is::<PointerLeft>() {
            view_state.set_hot(cx.window().is_hovered(view_state.id()));
        }

        view_state.prepare();

        let mut new_cx = cx.child();
        new_cx.transform *= view_state.transform;
        new_cx.view_state = view_state;

        f(&mut new_cx, event);

        view_state.prev_flags = view_state.flags;

        cx.view_state.propagate(view_state);
    }

    /// Layout a pod view.
    pub fn layout(
        view_state: &mut ViewState,
        cx: &mut LayoutCx,
        f: impl FnOnce(&mut LayoutCx) -> Size,
    ) -> Size {
        view_state.prepare_layout();

        let mut new_cx = cx.child();
        new_cx.view_state = view_state;

        let size = f(&mut new_cx);

        view_state.size = size;
        cx.view_state.propagate(view_state);

        size
    }

    /// Draw a pod view.
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
        canvas.view = None;

        // create the draw context
        let mut new_cx = cx.layer();
        new_cx.view_state = view_state;

        // draw the content
        f(&mut new_cx, &mut canvas);

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
        let (content, view_state) = Self::build(cx, |cx| self.view.build(cx, data));

        State {
            content,
            view_state,
        }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        if let Some(mut debug_tree) = cx.remove_context::<DebugTree>() {
            let child_tree = debug_tree.remove_or_new(0);
            cx.insert_context(child_tree);

            let start = Instant::now();

            Self::rebuild(&mut state.view_state, cx, |cx| {
                (self.view).rebuild(&mut state.content, cx, data, &old.view);
            });

            let time = start.elapsed();

            let mut child_tree = cx.remove_context::<DebugTree>().unwrap();
            child_tree.set_type::<V>();
            child_tree.set_rebuild_time(time);

            debug_tree.insert(0, child_tree);
            cx.insert_context(debug_tree);
        } else {
            Self::rebuild(&mut state.view_state, cx, |cx| {
                (self.view).rebuild(&mut state.content, cx, data, &old.view);
            });
        }
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        if let Some(mut debug_tree) = cx.remove_context::<DebugTree>() {
            let child_tree = debug_tree.remove_or_new(0);
            cx.insert_context(child_tree);

            let start = Instant::now();

            Self::event(&mut state.view_state, cx, event, |cx, event| {
                (self.view).event(&mut state.content, cx, data, event);
            });

            let time = start.elapsed();

            let mut child_tree = cx.remove_context::<DebugTree>().unwrap();
            child_tree.set_event_time(time);

            debug_tree.insert(0, child_tree);
            cx.insert_context(debug_tree);
        } else {
            Self::event(&mut state.view_state, cx, event, |cx, event| {
                (self.view).event(&mut state.content, cx, data, event);
            });
        }
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        if let Some(mut debug_tree) = cx.remove_context::<DebugTree>() {
            let child_tree = debug_tree.remove_or_new(0);
            cx.insert_context(child_tree);

            let start = Instant::now();

            let size = Self::layout(&mut state.view_state, cx, |cx| {
                (self.view).layout(&mut state.content, cx, data, space)
            });

            let time = start.elapsed();

            let mut child_tree = cx.remove_context::<DebugTree>().unwrap();
            child_tree.set_layout_time(time);
            child_tree.set_layout(DebugLayout {
                space,
                flex: state.view_state.flex(),
                tight: state.view_state.is_tight(),
            });

            debug_tree.insert(0, child_tree);
            cx.insert_context(debug_tree);

            size
        } else {
            Self::layout(&mut state.view_state, cx, |cx| {
                (self.view).layout(&mut state.content, cx, data, space)
            })
        }
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        if let Some(mut debug_tree) = cx.remove_context::<DebugTree>() {
            let child_tree = debug_tree.remove_or_new(0);
            cx.insert_context(child_tree);

            let start = Instant::now();

            Self::draw(&mut state.view_state, cx, canvas, |cx, canvas| {
                (self.view).draw(&mut state.content, cx, data, canvas);
            });

            let time = start.elapsed();

            let mut child_tree = cx.remove_context::<DebugTree>().unwrap();
            child_tree.set_draw_time(time);
            child_tree.set_draw(DebugDraw {
                rect: state.view_state.rect(),
                transform: state.view_state.transform(),
                depth: canvas.depth,
            });

            debug_tree.insert(0, child_tree);
            cx.insert_context(debug_tree);
        } else {
            Self::draw(&mut state.view_state, cx, canvas, |cx, canvas| {
                (self.view).draw(&mut state.content, cx, data, canvas);
            });
        }
    }
}
