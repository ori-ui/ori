use crate::{
    canvas::Canvas,
    event::{Event, PointerEvent},
    layout::{Point, Size, Space},
    rebuild::Rebuild,
    view::{BuildCx, Content, DrawCx, EventCx, LayoutCx, RebuildCx, State, View},
};

/// Create a new [`ClickHandler`], with an [`on_press`](ClickHandler::on_press()) callback.
pub fn on_press<T, V>(
    content: V,
    on_press: impl FnMut(&mut EventCx, &mut T) + 'static,
) -> ClickHandler<T, V> {
    ClickHandler::new(content).on_press(on_press)
}

/// Create a new [`ClickHandler`], with an [`on_release`](ClickHandler::on_release()) callback.
pub fn on_release<T, V>(
    content: V,
    on_release: impl FnMut(&mut EventCx, &mut T) + 'static,
) -> ClickHandler<T, V> {
    ClickHandler::new(content).on_release(on_release)
}

/// Create a new [`ClickHandler`], with an [`on_click`](ClickHandler::on_click()) callback.
pub fn on_click<T, V>(
    content: V,
    on_click: impl FnMut(&mut EventCx, &mut T) + 'static,
) -> ClickHandler<T, V> {
    ClickHandler::new(content).on_click(on_click)
}

/// A click handler.
#[derive(Rebuild)]
pub struct ClickHandler<T, V> {
    /// The content.
    pub content: Content<V>,
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

impl<T, V> ClickHandler<T, V> {
    const MAX_CLICK_DISTANCE: f32 = 10.0;

    /// Create a new [`ClickHandler`].
    pub fn new(content: V) -> Self {
        Self {
            content: Content::new(content),
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
pub struct ClickHandlerState {
    click_start: Point,
}

impl<T, V: View<T>> View<T> for ClickHandler<T, V> {
    type State = (ClickHandlerState, State<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        (ClickHandlerState::default(), self.content.build(cx, data))
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

        if event.is_handled() {
            return;
        }

        if let Some(pointer) = event.get::<PointerEvent>() {
            let local = cx.local(pointer.position);

            if cx.is_hot() && pointer.is_move() {
                event.handle();
            }

            if cx.is_hot() && pointer.is_press() {
                if let Some(on_press) = &mut self.on_press {
                    on_press(cx, data);
                    cx.request_rebuild();
                }

                state.click_start = local;

                cx.set_active(true);
                content.set_active(true);

                // FIXME: this isn't great
                cx.request_animation_frame();

                event.handle();
            } else if cx.is_active() && pointer.is_release() {
                cx.set_active(false);
                content.set_active(false);

                cx.request_animation_frame();

                if let Some(on_release) = &mut self.on_release {
                    on_release(cx, data);
                    cx.request_rebuild();
                }

                if local.distance(state.click_start) <= Self::MAX_CLICK_DISTANCE {
                    if let Some(on_click) = &mut self.on_click {
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
        self.content.draw(content, cx, data, canvas);
    }
}

/// A trait for building [`ClickHandler`]s.
pub trait BuildClickHandler<T, V, C> {
    /// Set the callback for when the button is pressed.
    fn on_press(self, cb: impl FnMut(&mut EventCx, &mut T) + 'static) -> ClickHandler<T, V>;

    /// Set the callback for when the button is released.
    fn on_release(self, cb: impl FnMut(&mut EventCx, &mut T) + 'static) -> ClickHandler<T, V>;

    /// Set the callback for when the button is clicked.
    fn on_click(self, cb: impl FnMut(&mut EventCx, &mut T) + 'static) -> ClickHandler<T, V>;
}

impl<T, V> BuildClickHandler<T, V, ()> for V {
    fn on_press(self, cb: impl FnMut(&mut EventCx, &mut T) + 'static) -> ClickHandler<T, V> {
        ClickHandler::new(self).on_press(cb)
    }

    fn on_release(self, cb: impl FnMut(&mut EventCx, &mut T) + 'static) -> ClickHandler<T, V> {
        ClickHandler::new(self).on_release(cb)
    }

    fn on_click(self, cb: impl FnMut(&mut EventCx, &mut T) + 'static) -> ClickHandler<T, V> {
        ClickHandler::new(self).on_click(cb)
    }
}

impl<T, V> BuildClickHandler<T, V, ClickHandler<T, V>> for ClickHandler<T, V> {
    fn on_press(self, cb: impl FnMut(&mut EventCx, &mut T) + 'static) -> ClickHandler<T, V> {
        self.on_press(cb)
    }

    fn on_release(self, cb: impl FnMut(&mut EventCx, &mut T) + 'static) -> ClickHandler<T, V> {
        self.on_release(cb)
    }

    fn on_click(self, cb: impl FnMut(&mut EventCx, &mut T) + 'static) -> ClickHandler<T, V> {
        self.on_click(cb)
    }
}
