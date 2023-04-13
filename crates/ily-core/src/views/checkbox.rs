use glam::Vec2;
use ily_graphics::{Quad, TextAlign, TextSection};

use crate::{
    Bindable, BoxConstraints, DrawContext, Event, EventContext, LayoutContext, PointerEvent, Scope,
    SharedSignal, Signal, View, ViewState,
};

#[derive(Default)]
pub struct Checkbox {
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
            if checked.get_untracked() == binding.get() {
                return;
            }

            checked.set(*binding.get());
        });

        let checked = self.checked.clone();
        cx.effect(move || {
            if checked.get() == binding.get_untracked() {
                return;
            }

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
        let checked = self.checkbox.checked.downgrade();
        cx.effect(move || {
            if checked.get_untracked() == Some(binding.get()) {
                return;
            }

            checked.set(*binding.get());
        });

        let checked = self.checkbox.checked.downgrade();
        cx.effect(move || {
            if checked.get() == Some(binding.get_untracked()) {
                return;
            }

            if let Some(checked) = checked.get() {
                binding.set(*checked);
            }
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

    fn build(&self) -> ViewState<Self::State> {
        ViewState::new((), Some("checkbox"))
    }

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

    fn layout(&self, _state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        cx.state.active = self.checked.cloned_untracked();

        let width = cx.style_range("width", bc.width());
        let height = cx.style_range("height", bc.height());
        bc.constrain(Vec2::new(width, height))
    }

    fn draw(&self, _state: &mut Self::State, cx: &mut DrawContext) {
        cx.state.active = self.checked.cloned_untracked();

        let color = cx.style("color");
        let background = cx.style("background");
        let border_color = cx.style("border-color");

        let border_radius = cx.style_range("border-radius", 0.0..20.0);
        let border_width = cx.style_range("border-width", 0.0..20.0);

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
                scale: cx.rect().size().min_element() * 0.8,
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
