use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
};

/// A single UI component.
///
/// This trait is implemented by all UI components. The user interface is built
/// by composing these components into a `view-tree`. This operation should be
/// fast, as it is performed very often.
///
/// A view also has an associated `state` type, that is persistent across `view-trees`.
/// When calling [`View::build`], the view will build it's state. A view containing
/// another view must also store it's child's state. This is usually done by wrapping
/// it in a tuple `(MyState, State)`.
///
/// In case a view contains another view the contents should always be wrapped in
/// either [`PodState`] or [`SeqState`]. If this is not done strange issues
/// are _very_ likely to occur.
///
/// For information on styling see [`style`].
///
/// [`View`] has four primary methods:
/// - [`View::rebuild`] is called after a new `view-tree` has been built, on the
///     new tree. The view can then compare itself to the old tree and update it's
///     state accordingly. When a view differs from the old tree, it should call
///     [`RebuildCx::layout`] or [`RebuildCx::draw`] when applicable.
///     This can be quite tedius to write out, so the [`Rebuild`] derive macro can be
///     used to generate this code.
/// - [`View::event`] is called when an event occurs. The should then handle the
///     event and return whether it handled it. Command events can be send using [`BaseCx::cmd`].
/// - [`View::layout`] is called when the view needs to be laid out. A leaf view
///     should compute it's own size in accordance with the given [`Space`], and
///     return it. A container view should pass an appropriate [`Space`] to it's
///     contents and the compute it's own size based on the contents' size(s).
/// - [`View::draw`] is called when the view needs to be drawn.
///
/// For examples see the implementation of views like [`Button`] or [`Checkbox`].
///
/// [`BaseCx::cmd`]: crate::context::BaseCx::cmd
/// [`PodState`]: super::PodState
/// [`SeqState`]: super::SeqState
/// [`ViewState`]: super::ViewState
/// [`Rebuild`]: crate::rebuild::Rebuild
/// [`Button`]: crate::views::Button
/// [`Checkbox`]: crate::views::Checkbox
/// [`style`]: crate::style
pub trait View<T: ?Sized = ()> {
    /// The state of the view, see top-level documentation for more information.
    type State;

    /// Build the view state, see top-level documentation for more information.
    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State;

    /// Rebuild the view state, see top-level documentation for more information.
    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self);

    /// Handle an event, see top-level documentation for more information.
    #[must_use]
    fn event(
        &mut self,
        state: &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool;

    /// Layout the view, see top-level documentation for more information.
    #[must_use]
    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size;

    /// Draw the view, see top-level documentation for more information.
    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T);
}

impl<T, V: View<T>> View<T> for Option<V> {
    type State = Option<V::State>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        self.as_mut().map(|view| view.build(cx, data))
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        if let Some(view) = self {
            if state.is_none() {
                *state = Some(view.build(&mut cx.as_build_cx(), data));
            }

            if let Some(old_view) = old {
                view.rebuild(state.as_mut().unwrap(), cx, data, old_view);
            }
        } else {
            *state = None;
        }
    }

    fn event(
        &mut self,
        state: &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        match self {
            Some(view) => view.event(state.as_mut().unwrap(), cx, data, event),
            None => false,
        }
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        if let Some(view) = self {
            view.layout(state.as_mut().unwrap(), cx, data, space)
        } else {
            space.min
        }
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        if let Some(view) = self {
            view.draw(state.as_mut().unwrap(), cx, data);
        }
    }
}

impl<T> View<T> for () {
    type State = ();

    fn build(&mut self, _cx: &mut BuildCx, _data: &mut T) -> Self::State {}

    fn rebuild(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut RebuildCx,
        _data: &mut T,
        _old: &Self,
    ) {
    }

    fn event(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut EventCx,
        _data: &mut T,
        _event: &Event,
    ) -> bool {
        false
    }

    fn layout(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        space.min
    }

    fn draw(&mut self, _state: &mut Self::State, _cx: &mut DrawCx, _data: &mut T) {}
}

impl<T, V: View<T>, E: View<T>> View<T> for Result<V, E> {
    type State = Result<V::State, E::State>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        match self {
            Ok(view) => Ok(view.build(cx, data)),
            Err(view) => Err(view.build(cx, data)),
        }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        match (&mut *self, &mut *state, old) {
            (Ok(view), Ok(state), Ok(old)) => view.rebuild(state, cx, data, old),
            (Err(view), Err(state), Err(old)) => view.rebuild(state, cx, data, old),
            _ => {
                *state = self.build(&mut cx.as_build_cx(), data);
                *cx.view_state = Default::default();

                cx.layout();
            }
        }
    }

    fn event(
        &mut self,
        state: &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        match (self, state) {
            (Ok(view), Ok(state)) => view.event(state, cx, data, event),
            (Err(view), Err(state)) => view.event(state, cx, data, event),
            _ => false,
        }
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        match (self, state) {
            (Ok(view), Ok(state)) => view.layout(state, cx, data, space),
            (Err(view), Err(state)) => view.layout(state, cx, data, space),
            _ => space.min,
        }
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        match (self, state) {
            (Ok(view), Ok(state)) => view.draw(state, cx, data),
            (Err(view), Err(state)) => view.draw(state, cx, data),
            _ => {}
        }
    }
}
