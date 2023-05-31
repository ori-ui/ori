use glam::Vec2;
use ori_macro::Build;
use ori_reactive::{CallbackEmitter, Event, Scope};

use crate::{
    AvailableSpace, BindCallback, Children, Context, DrawContext, EventContext, FlexLayout,
    LayoutContext, PointerEvent, Style, View,
};

/// A div view.
#[derive(Default, Build)]
pub struct Div {
    /// On click callback.
    #[event]
    pub on_click: CallbackEmitter<PointerEvent>,
    /// On release callback.
    #[event]
    pub on_release: CallbackEmitter<PointerEvent>,
    /// The children of the div.
    #[children]
    pub children: Children,
}

impl Div {
    /// Create a new div.
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind the on click callback.
    pub fn on_press(
        mut self,
        cx: Scope,
        callback: impl FnMut(&PointerEvent) + Send + 'static,
    ) -> Self {
        self.on_click.bind(cx, callback);

        self
    }

    /// Bind the on release callback.
    pub fn on_release(
        mut self,
        cx: Scope,
        callback: impl FnMut(&PointerEvent) + Send + 'static,
    ) -> Self {
        self.on_release.bind(cx, callback);

        self
    }

    fn handle_pointer_event(
        &self,
        cx: &mut EventContext,
        event: &PointerEvent,
        handled: bool,
    ) -> bool {
        if event.is_press() && cx.hovered() && !handled {
            if !self.on_click.is_empty() {
                cx.activate();
                self.on_click.emit(event);
            }
        } else if event.is_release() && cx.state.active {
            cx.deactivate();

            if !self.on_release.is_empty() {
                self.on_release.emit(event);
            }
        } else {
            return false;
        }

        true
    }
}

impl View for Div {
    type State = ();

    fn build(&self) -> Self::State {}

    fn style(&self) -> Style {
        Style::new("div")
    }

    fn event(&self, _: &mut Self::State, cx: &mut EventContext, event: &Event) {
        self.children.event(cx, event);

        if let Some(pointer_event) = event.get::<PointerEvent>() {
            if self.handle_pointer_event(cx, pointer_event, event.is_handled()) {
                event.handle();
            }
        }
    }

    fn layout(&self, _: &mut Self::State, cx: &mut LayoutContext, space: AvailableSpace) -> Vec2 {
        let flex = FlexLayout::from_style(cx);
        space.constrain(self.children.flex_layout(cx, space, flex))
    }

    fn draw(&self, _: &mut Self::State, cx: &mut DrawContext) {
        cx.draw_quad();

        cx.draw_layer(|cx| {
            self.children.draw(cx);
        });
    }
}
