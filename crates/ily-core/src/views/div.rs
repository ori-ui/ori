use glam::Vec2;
use ily_macro::Build;

use crate::{
    Axis, BoxConstraints, Children, Context, DrawContext, Event, EventContext, EventSignal,
    FlexLayout, LayoutContext, Node, Parent, PointerEvent, Scope, Sendable, Style, View,
};

#[derive(Default, Build)]
pub struct Div {
    #[event]
    pub on_event: Option<EventSignal<Event>>,
    #[event]
    pub on_press: Option<EventSignal<PointerEvent>>,
    #[event]
    pub on_release: Option<EventSignal<PointerEvent>>,
    pub children: Children,
}

impl Div {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn child(mut self, child: impl View) -> Self {
        self.add_child(child);
        self
    }

    pub fn on_event<'a>(
        mut self,
        cx: Scope<'a>,
        callback: impl FnMut(&Event) + Sendable + 'a,
    ) -> Self {
        self.on_event
            .get_or_insert_with(|| EventSignal::new())
            .subscribe(cx, callback);

        self
    }

    pub fn on_press<'a>(
        mut self,
        cx: Scope<'a>,
        callback: impl FnMut(&PointerEvent) + Sendable + 'a,
    ) -> Self {
        self.on_press
            .get_or_insert_with(|| EventSignal::new())
            .subscribe(cx, callback);

        self
    }

    pub fn on_release<'a>(
        mut self,
        cx: Scope<'a>,
        callback: impl FnMut(&PointerEvent) + Sendable + 'a,
    ) -> Self {
        self.on_release
            .get_or_insert_with(|| EventSignal::new())
            .subscribe(cx, callback);

        self
    }

    fn handle_pointer_event(&self, cx: &mut EventContext, event: &PointerEvent) -> bool {
        if event.is_press() && cx.hovered() {
            if let Some(on_press) = &self.on_press {
                cx.activate();
                on_press.emit(event.clone());
            }
        } else if event.is_release() && cx.state.active {
            cx.deactivate();

            if let Some(on_release) = &self.on_release {
                on_release.emit(event.clone());
            }
        } else {
            return false;
        }

        true
    }
}

impl Parent for Div {
    fn add_child(&mut self, child: impl View) {
        self.children.push(Node::new(child));
    }
}

impl View for Div {
    type State = ();

    fn build(&self) -> Self::State {}

    fn style(&self) -> Style {
        Style::new("div")
    }

    fn event(&self, _state: &mut Self::State, cx: &mut EventContext, event: &Event) {
        for child in &self.children {
            child.event(cx, event);
        }

        if let Some(on_event) = &self.on_event {
            on_event.emit(event.clone());
        }

        if event.is_handled() {
            return;
        }

        if let Some(pointer_event) = event.get::<PointerEvent>() {
            if self.handle_pointer_event(cx, pointer_event) {
                event.handle();
            }
        }
    }

    fn layout(&self, _state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        let bc = cx.style_constraints(bc);

        let axis = cx.style::<Axis>("direction");
        let padding = cx.style_range("padding", 0.0..bc.max.min_element() / 2.0);
        let gap = cx.style_range("gap", 0.0..axis.major(bc.max));

        let justify_content = cx.style("justify-content");
        let align_items = cx.style("align-items");

        let flex = FlexLayout {
            axis,
            justify_content,
            align_items,
            gap,
            offset: Vec2::splat(padding),
        };

        let content_bc = bc.shrink(Vec2::splat(padding * 2.0));
        let size = self.children.flex_layout(cx, content_bc, flex);

        size + Vec2::splat(padding * 2.0)
    }

    fn draw(&self, _state: &mut Self::State, cx: &mut DrawContext) {
        cx.draw_quad();

        cx.draw_layer(|cx| {
            for child in &self.children {
                child.draw(cx);
            }
        });
    }
}
