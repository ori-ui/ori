use glam::Vec2;
use ily_graphics::{Color, Quad, TextAlign, TextSection};

use crate::{
    attributes, Bindable, BoxConstraints, DrawContext, Event, EventContext, LayoutContext, Length,
    PointerEvent, Scope, SharedSignal, Signal, View,
};

#[derive(Default)]
pub struct Checkbox {
    color: Option<Color>,
    background: Option<Color>,
    border_radius: Option<Length>,
    border_width: Option<Length>,
    border_color: Option<Color>,
    width: Option<Length>,
    height: Option<Length>,
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
        let checked = self.checked.clone();
        cx.effect(move || {
            checked.set(*binding.get());
        });

        let checked = self.checked.clone();
        cx.effect(move || {
            binding.set(*checked.get());
        });

        self
    }
}

pub struct CheckboxBinding<'a> {
    checkbox: &'a mut Checkbox,
}

impl<'a> CheckboxBinding<'a> {
    pub fn checked<'b>(&self, cx: Scope<'b>, binding: &'b Signal<bool>) {
        let checked = self.checkbox.checked.clone();
        cx.effect(move || {
            checked.set(*binding.get());
        });

        let checked = self.checkbox.checked.clone();
        cx.effect(move || {
            binding.set(*checked.get());
        });
    }
}

impl Bindable for Checkbox {
    type Setter<'a> = CheckboxBinding<'a>;

    fn setter(&mut self) -> Self::Setter<'_> {
        CheckboxBinding { checkbox: self }
    }
}

impl View for Checkbox {
    type State = ();

    fn build(&self) -> Self::State {}

    fn element(&self) -> Option<&'static str> {
        Some("checkbox")
    }

    fn event(&self, _: &mut Self::State, cx: &mut EventContext, event: &Event) {
        if event.is_handled() {
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

    fn layout(&self, _state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        attributes! {
            cx, self,
            width: "width",
            height: "height",
        }

        let width = width.pixels();
        let height = height.pixels();

        bc.constrain(Vec2::new(width, height))
    }

    fn draw(&self, _state: &mut Self::State, cx: &mut DrawContext) {
        attributes! {
            cx, self,
            color: "color",
            background: "background",
            border_radius: "border-radius",
            border_width: "border-width",
            border_color: "border-color",
            width: "width",
            height: "height",
        }

        let border_radius = border_radius.pixels();
        let border_width = border_width.pixels();
        let width = width.pixels();
        let height = height.pixels();

        let quad = Quad {
            rect: cx.rect(),
            background,
            border_radius: [border_radius; 4],
            border_width,
            border_color,
        };

        cx.draw_primitive(quad);

        if *self.checked.get() {
            let section = TextSection {
                position: cx.rect().center(),
                bounds: cx.rect().size(),
                scale: f32::min(width, height) * 0.8,
                h_align: TextAlign::Center,
                v_align: TextAlign::Center,
                text: String::from(Self::CHECKMARK),
                font: Some(String::from("MaterialIcons-Regular")),
                color,
            };

            cx.draw_primitive(section);
        }
    }
}
