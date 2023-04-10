use glam::Vec2;
use ily_graphics::{Color, Quad};
use ily_reactive::{EventSignal, Scope};

use crate::{BoxConstraints, Event, PaintContext, PointerButton, PointerPress, View};

pub struct Button {
    on_press: EventSignal<PointerPress>,
}

impl Button {
    pub fn new() -> Self {
        Self {
            on_press: EventSignal::new(),
        }
    }

    pub fn on_press<'a>(self, cx: Scope<'a>, callback: impl FnMut(&PointerPress) + 'a) -> Self {
        self.on_press.subscribe(cx, callback);

        self
    }
}

impl View for Button {
    fn classes(&self) -> Vec<String> {
        vec!["button".to_string()]
    }

    fn layout(&self, _bc: BoxConstraints) -> Vec2 {
        Vec2::new(100.0, 50.0)
    }

    fn event(&self, event: &Event) {
        if let Some(pointer_press) = event.get::<PointerPress>() {
            if pointer_press.button == PointerButton::Primary {
                self.on_press.emit(pointer_press.clone());
            }
        }
    }

    fn paint(&self, cx: &mut PaintContext) {
        let quad = Quad {
            rect: cx.rect(),
            background: Color::GREEN,
            ..Default::default()
        };

        cx.draw_primitive(quad);
    }
}
