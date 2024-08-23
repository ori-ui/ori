use std::marker::PhantomData;

use ori_macro::Build;

use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::{Event, PointerButton},
    layout::{Size, Space},
    rebuild::Rebuild,
    view::{Pod, State, View},
};

/// Create a new [`Clickable`], that calls `on_press` when pressed.
pub fn on_press<T, V, F>(content: V, on_press: F) -> Clickable<T, V, F>
where
    V: View<T>,
    F: FnMut(&mut EventCx, &mut T) + 'static,
{
    Clickable::new(content, ClickEvent::Press, on_press)
}

/// Create a new [`Clickable`], that calls `on_release` when released.
pub fn on_release<T, V, F>(content: V, on_release: F) -> Clickable<T, V, F>
where
    V: View<T>,
    F: FnMut(&mut EventCx, &mut T) + 'static,
{
    Clickable::new(content, ClickEvent::Release, on_release)
}

/// Create a new [`Clickable`], that calls `on_click` when clicked.
pub fn on_click<T, V, F>(content: V, on_click: F) -> Clickable<T, V, F>
where
    V: View<T>,
    F: FnMut(&mut EventCx, &mut T) + 'static,
{
    Clickable::new(content, ClickEvent::Click, on_click)
}

/// A click event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClickEvent {
    /// The press event.
    Press,

    /// The release event.
    Release,

    /// The click event.
    Click,
}

/// A click handler.
#[derive(Build, Rebuild)]
pub struct Clickable<T, V, F>
where
    V: View<T>,
    F: FnMut(&mut EventCx, &mut T) + 'static,
{
    /// The content.
    pub content: Pod<V>,

    /// Whether the item should be clickable when it's descendants are clicked.
    ///
    /// Defaults to `true`.
    pub descendants: bool,

    /// The button to listen for.
    ///
    /// If `Some` the callbacks will only be called when this button is pressed.
    pub button: Option<PointerButton>,

    /// The event to listen for.
    pub event: ClickEvent,

    /// The callback.
    #[build(ignore)]
    pub callback: F,

    marker: PhantomData<fn() -> T>,
}

impl<T, V, F> Clickable<T, V, F>
where
    V: View<T>,
    F: FnMut(&mut EventCx, &mut T) + 'static,
{
    /// Create a new [`Clickable`].
    pub fn new(content: V, event: ClickEvent, callback: F) -> Self {
        Self {
            content: Pod::new(content),
            descendants: true,
            button: None,
            event,
            callback,
            marker: PhantomData,
        }
    }

    fn is_button(&self, button: PointerButton) -> bool {
        self.button.map_or(true, |b| b == button)
    }
}

impl<T, V, F> View<T> for Clickable<T, V, F>
where
    V: View<T>,
    F: FnMut(&mut EventCx, &mut T) + 'static,
{
    type State = State<T, V>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        self.content.build(cx, data)
    }

    fn rebuild(&mut self, content: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);

        self.content.rebuild(content, cx, data, &old.content);
    }

    fn event(&mut self, content: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        let is_hot = content.is_hot() || (content.has_hot() && self.descendants);

        match event {
            Event::PointerPressed(e) if is_hot && self.is_button(e.button) => {
                if self.event == ClickEvent::Press {
                    (self.callback)(cx, data);
                    cx.request_rebuild();
                }

                content.set_active(true);
            }
            Event::PointerReleased(e) if content.is_active() && self.is_button(e.button) => {
                if self.event == ClickEvent::Release {
                    (self.callback)(cx, data);
                    cx.request_rebuild();
                }

                if e.clicked && self.event == ClickEvent::Click {
                    (self.callback)(cx, data);
                    cx.request_rebuild();
                }

                content.set_active(false);
            }
            _ => {}
        }

        self.content.event(content, cx, data, event);
    }

    fn layout(
        &mut self,
        content: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        self.content.layout(content, cx, data, space)
    }

    fn draw(&mut self, content: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        self.content.draw(content, cx, data);
    }
}
