use ori_macro::Build;

use crate::{
    canvas::Canvas,
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::{Event, PointerButton},
    layout::{Size, Space},
    rebuild::Rebuild,
    view::{Pod, State, View},
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
#[derive(Build, Rebuild)]
pub struct Clickable<T, V> {
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

    /// The callback for when the button is pressed.
    #[build(ignore)]
    #[allow(clippy::type_complexity)]
    pub on_press: Option<Box<dyn FnMut(&mut EventCx, &mut T) + 'static>>,

    /// The callback for when the button is released.
    #[build(ignore)]
    #[allow(clippy::type_complexity)]
    pub on_release: Option<Box<dyn FnMut(&mut EventCx, &mut T) + 'static>>,

    /// The callback for when the button is clicked.
    #[build(ignore)]
    #[allow(clippy::type_complexity)]
    pub on_click: Option<Box<dyn FnMut(&mut EventCx, &mut T) + 'static>>,
}

impl<T, V> Clickable<T, V> {
    /// Create a new [`Clickable`].
    pub fn new(content: V) -> Self {
        Self {
            content: Pod::new(content),
            descendants: true,
            button: None,
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

    fn is_button(&self, button: PointerButton) -> bool {
        self.button.map_or(true, |b| b == button)
    }
}

impl<T, V: View<T>> View<T> for Clickable<T, V> {
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
                if let Some(ref mut on_press) = self.on_press {
                    on_press(cx, data);
                    cx.request_rebuild();
                }

                content.set_active(true);
            }
            Event::PointerReleased(e) if content.is_active() && self.is_button(e.button) => {
                if let Some(ref mut on_release) = self.on_release {
                    on_release(cx, data);
                    cx.request_rebuild();
                }

                if e.clicked {
                    if let Some(ref mut on_click) = self.on_click {
                        on_click(cx, data);
                        cx.request_rebuild();
                    }
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

    fn draw(
        &mut self,
        content: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        canvas.set_hoverable(cx.id());
        self.content.draw(content, cx, data, canvas);
    }
}
