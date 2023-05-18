use glam::Vec2;
use ori_macro::Build;

use crate::{
    Axis, BindCallback, BoxConstraints, CallbackEmitter, Children, Context, DrawContext, Event,
    EventContext, FlexLayout, LayoutContext, PointerEvent, Scope, Sendable, Style, View,
};

#[derive(Default, Build)]
pub struct Div {
    #[event]
    pub on_press: CallbackEmitter<PointerEvent>,
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
        callback: impl FnMut(&PointerEvent) + Sendable + 'static,
    ) -> Self {
        self.on_press.bind(cx, callback);

        self
    }

    pub fn on_release<'a>(
        mut self,
        cx: Scope,
        callback: impl FnMut(&PointerEvent) + Sendable + 'static,
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
            if !self.on_press.is_empty() {
                cx.activate();
                self.on_press.emit(event);
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

    #[tracing::instrument(name = "Div", skip(self, cx, bc))]
    fn layout(&self, _: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        let axis = cx.style::<Axis>("direction");
        let gap = cx.style_range("gap", 0.0..axis.major(bc.max));

        let justify_content = cx.style("justify-content");
        let align_items = cx.style("align-items");

        let flex = FlexLayout {
            axis,
            justify_content,
            align_items,
            gap,
            ..Default::default()
        };

        self.children.flex_layout(cx, bc, flex)
    }

    #[tracing::instrument(name = "Div", skip(self, cx))]
    fn draw(&self, _: &mut Self::State, cx: &mut DrawContext) {
        cx.draw_quad();

        cx.draw_layer(|cx| {
            for child in &self.children {
                child.draw(cx);
            }
        });
    }
}