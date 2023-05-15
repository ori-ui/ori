use glam::Vec2;
use ori_graphics::{TextAlign, TextSection};
use ori_macro::Build;

use crate::{
    BoxConstraints, Context, DrawContext, Event, EventContext, LayoutContext, PointerEvent, Scope,
    SharedSignal, Signal, Style, View,
};

#[derive(Default, Build)]
pub struct Checkbox {
    #[bind]
    checked: SharedSignal<bool>,
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

    pub fn bind_checked<'a>(self, cx: Scope<'a>, binding: &'a Signal<bool>) -> Self {
        let signal = cx.alloc(self.checked.clone());
        cx.bind(signal, binding);
        self
    }
}

impl View for Checkbox {
    type State = ();

    fn build(&self) -> Self::State {}

    fn style(&self) -> Style {
        Style::new("checkbox")
    }

    #[tracing::instrument(name = "Checkbox", skip(self, cx, event))]
    fn event(&self, _: &mut Self::State, cx: &mut EventContext, event: &Event) {
        cx.state.active = self.checked.cloned_untracked();

        if event.is_handled() || !cx.hovered() {
            return;
        }

        if let Some(pointer_event) = event.get::<PointerEvent>() {
            if pointer_event.is_press() {
                self.checked.set(!self.checked.cloned());
                event.handle();
                cx.request_redraw();
            }
        }
    }

    #[tracing::instrument(name = "Checkbox", skip(self, cx, bc))]
    fn layout(&self, _: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        cx.state.active = self.checked.cloned_untracked();

        let width = cx.style_range("width", bc.width());
        let height = cx.style_range("height", bc.height());
        bc.constrain(Vec2::new(width, height))
    }

    #[tracing::instrument(name = "Checkbox", skip(self, cx))]
    fn draw(&self, _: &mut Self::State, cx: &mut DrawContext) {
        cx.state.active = self.checked.cloned_untracked();

        cx.draw_quad();

        if *self.checked.get() {
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
