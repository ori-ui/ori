use crate::{
    canvas::Canvas,
    event::{ActiveChanged, Event, PointerEvent},
    layout::{Point, Size, Space},
    rebuild::Rebuild,
    view::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View},
};

/// Create a new [`Clickable`].
pub fn clickable<T, V>(content: V) -> Clickable<T, V> {
    Clickable::new(content)
}

/// Create a new [`Clickable`], with an [`on_press`](Clickable::on_press()) callback.
pub fn on_press<T, V>(
    content: V,
    on_press: impl FnMut(&mut EventCx, &mut T) + 'static,
) -> Clickable<T, V> {
    clickable(content).on_press(on_press)
}

/// Create a new [`Clickable`], with an [`on_release`](Clickable::on_release()) callback.
pub fn on_release<T, V>(
    content: V,
    on_release: impl FnMut(&mut EventCx, &mut T) + 'static,
) -> Clickable<T, V> {
    clickable(content).on_release(on_release)
}

/// Create a new [`Clickable`], with an [`on_click`](Clickable::on_click()) callback.
pub fn on_click<T, V>(
    content: V,
    on_click: impl FnMut(&mut EventCx, &mut T) + 'static,
) -> Clickable<T, V> {
    clickable(content).on_click(on_click)
}

/// A click handler.
#[derive(Rebuild)]
pub struct Clickable<T, V> {
    /// The content.
    pub content: V,
    /// The callback for when the button is pressed.
    #[allow(clippy::type_complexity)]
    pub on_press: Option<Box<dyn FnMut(&mut EventCx, &mut T) + 'static>>,
    /// The callback for when the button is released.
    #[allow(clippy::type_complexity)]
    pub on_release: Option<Box<dyn FnMut(&mut EventCx, &mut T) + 'static>>,
    /// The callback for when the button is clicked.
    #[allow(clippy::type_complexity)]
    pub on_click: Option<Box<dyn FnMut(&mut EventCx, &mut T) + 'static>>,
}

impl<T, V> Clickable<T, V> {
    const MAX_CLICK_DISTANCE: f32 = 10.0;

    /// Create a new [`Clickable`].
    pub fn new(content: V) -> Self {
        Self {
            content,
            on_press: None,
            on_release: None,
            on_click: None,
        }
    }

    /// Set the callback for when the button is pressed.
    pub fn on_press(mut self, on_press: impl FnMut(&mut EventCx, &mut T) + 'static) -> Self {
        self.on_press = Some(Box::new(on_press));
        self
    }

    /// Set the callback for when the button is released.
    pub fn on_release(mut self, on_release: impl FnMut(&mut EventCx, &mut T) + 'static) -> Self {
        self.on_release = Some(Box::new(on_release));
        self
    }

    /// Set the callback for when the button is clicked.
    pub fn on_click(mut self, on_click: impl FnMut(&mut EventCx, &mut T) + 'static) -> Self {
        self.on_click = Some(Box::new(on_click));
        self
    }
}

#[doc(hidden)]
#[derive(Default)]
pub struct ClickableState {
    click_start: Point,
}

impl<T, V: View<T>> View<T> for Clickable<T, V> {
    type State = (ClickableState, V::State);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        (ClickableState::default(), self.content.build(cx, data))
    }

    fn rebuild(
        &mut self,
        (_state, content): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        Rebuild::rebuild(self, cx, old);

        self.content.rebuild(content, cx, data, &old.content);
    }

    fn event(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) {
        self.content.event(content, cx, data, event);

        if let Some(pointer) = event.get::<PointerEvent>() {
            if pointer.is_press() {
                state.click_start = pointer.position;
            }

            if pointer.is_press() && cx.is_hot() {
                if let Some(ref mut on_press) = self.on_press {
                    on_press(cx, data);
                    cx.request_rebuild();
                }

                cx.set_active(true);
                let event = Event::new_non_propagating(ActiveChanged(true));
                self.content.event(content, cx, data, &event);
            }

            if pointer.is_release() && cx.is_active() {
                if let Some(ref mut on_release) = self.on_release {
                    on_release(cx, data);
                    cx.request_rebuild();
                }

                cx.set_active(false);
                let event = Event::new_non_propagating(ActiveChanged(false));
                self.content.event(content, cx, data, &event);

                let click_distance = (pointer.position - state.click_start).length();
                if click_distance <= Self::MAX_CLICK_DISTANCE {
                    if let Some(ref mut on_click) = self.on_click {
                        on_click(cx, data);
                        cx.request_rebuild();
                    }
                }
            }
        }
    }

    fn layout(
        &mut self,
        (_state, content): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        self.content.layout(content, cx, data, space)
    }

    fn draw(
        &mut self,
        (_state, content): &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        canvas.view(cx.id());
        self.content.draw(content, cx, data, canvas);
    }
}

/// A trait for building [`Clickable`]s.
pub trait BuildClickable<T, V, C> {
    /// Set the callback for when the button is pressed.
    fn on_press(self, cb: impl FnMut(&mut EventCx, &mut T) + 'static) -> Clickable<T, V>;

    /// Set the callback for when the button is released.
    fn on_release(self, cb: impl FnMut(&mut EventCx, &mut T) + 'static) -> Clickable<T, V>;

    /// Set the callback for when the button is clicked.
    fn on_click(self, cb: impl FnMut(&mut EventCx, &mut T) + 'static) -> Clickable<T, V>;
}

impl<T, V> BuildClickable<T, V, ()> for V {
    fn on_press(self, cb: impl FnMut(&mut EventCx, &mut T) + 'static) -> Clickable<T, V> {
        Clickable::new(self).on_press(cb)
    }

    fn on_release(self, cb: impl FnMut(&mut EventCx, &mut T) + 'static) -> Clickable<T, V> {
        Clickable::new(self).on_release(cb)
    }

    fn on_click(self, cb: impl FnMut(&mut EventCx, &mut T) + 'static) -> Clickable<T, V> {
        Clickable::new(self).on_click(cb)
    }
}

impl<T, V> BuildClickable<T, V, Clickable<T, V>> for Clickable<T, V> {
    fn on_press(self, cb: impl FnMut(&mut EventCx, &mut T) + 'static) -> Clickable<T, V> {
        self.on_press(cb)
    }

    fn on_release(self, cb: impl FnMut(&mut EventCx, &mut T) + 'static) -> Clickable<T, V> {
        self.on_release(cb)
    }

    fn on_click(self, cb: impl FnMut(&mut EventCx, &mut T) + 'static) -> Clickable<T, V> {
        self.on_click(cb)
    }
}
