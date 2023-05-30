use glam::Vec2;
use ori_macro::Build;
use ori_reactive::{CallbackEmitter, Event, Scope};

use crate::{
    AvailableSpace, BindCallback, Children, Context, DrawContext, EventContext, FlexLayout,
    LayoutContext, PointerEvent, Style, View,
};

#[derive(Default, Build)]
pub struct Div {
    #[event]
    pub on_click: CallbackEmitter<PointerEvent>,
    #[event]
    pub on_release: CallbackEmitter<PointerEvent>,
    #[children]
    pub children: Children,
}

impl Div {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_press<'a>(
        mut self,
        cx: Scope,
        callback: impl FnMut(&PointerEvent) + Send + 'static,
    ) -> Self {
        self.on_click.bind(cx, callback);

        self
    }

    pub fn on_release<'a>(
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

    #[tracing::instrument(name = "Div", skip(self, cx, event))]
    fn event(&self, _: &mut Self::State, cx: &mut EventContext, event: &Event) {
        for child in &self.children {
            child.event(cx, event);
        }

        if let Some(pointer_event) = event.get::<PointerEvent>() {
            if self.handle_pointer_event(cx, pointer_event, event.is_handled()) {
                event.handle();
            }
        }
    }

    #[tracing::instrument(name = "Div", skip(self, cx, space))]
    fn layout(&self, _: &mut Self::State, cx: &mut LayoutContext, space: AvailableSpace) -> Vec2 {
        let flex = FlexLayout::from_style(cx);
        space.constrain(self.children.flex_layout(cx, space, flex))
    }

    #[tracing::instrument(name = "Div", skip(self, cx))]
    fn draw(&self, _: &mut Self::State, cx: &mut DrawContext) {
        cx.draw_quad();

        cx.layer().draw(|cx| {
            self.children.draw(cx);
        });
    }
}
