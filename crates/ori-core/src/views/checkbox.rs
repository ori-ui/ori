use glam::Vec2;
use ori_graphics::{TextAlign, TextSection};
use ori_macro::Build;
use ori_reactive::{CallbackEmitter, Event, OwnedSignal, Scope, Signal};

use crate::{
    AvailableSpace, Context, DrawContext, EventContext, LayoutContext, PointerEvent, Style, View,
};

#[derive(Default, Build)]
pub struct Checkbox {
    #[prop]
    #[bind]
    checked: OwnedSignal<bool>,
    #[event]
    on_click: CallbackEmitter<PointerEvent>,
}

impl Checkbox {
    const CHECKMARK: &'static str = "\u{e876}";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn checked(self, checked: bool) -> Self {
        self.checked.set(checked);
        self
    }

    pub fn bind_checked(mut self, _cx: Scope, binding: Signal<bool>) -> Self {
        self.checked.bind(binding);
        self
    }
}

impl View for Checkbox {
    type State = ();

    fn build(&self) -> Self::State {}

    fn style(&self) -> Style {
        Style::new("checkbox")
    }

    fn event(&self, _: &mut Self::State, cx: &mut EventContext, event: &Event) {
        cx.state.active = self.checked.get();

        if event.is_handled() || !cx.hovered() {
            return;
        }

        if let Some(pointer_event) = event.get::<PointerEvent>() {
            if pointer_event.is_press() {
                self.checked.set(!self.checked.get());
                self.on_click.emit(pointer_event);
                event.handle();
                cx.request_redraw();
            }
        }
    }

    fn layout(&self, _: &mut Self::State, cx: &mut LayoutContext, space: AvailableSpace) -> Vec2 {
        cx.state.active = self.checked.get();

        let width = cx.style_range("width", space.x_axis());
        let height = cx.style_range("height", space.y_axis());
        space.constrain(Vec2::new(width, height))
    }

    fn draw(&self, _: &mut Self::State, cx: &mut DrawContext) {
        cx.state.active = self.checked.get();

        cx.draw_quad();

        if self.checked.get() {
            let section = TextSection {
                rect: cx.rect(),
                scale: cx.rect().size().min_element() * 0.8,
                h_align: TextAlign::Center,
                v_align: TextAlign::Center,
                text: String::from(Self::CHECKMARK),
                font_family: Some(String::from("Material Icons")),
                color: cx.style("color"),
                ..Default::default()
            };

            cx.draw(section);
        }
    }
}
