use glam::Vec2;
use ily_graphics::{Color, Quad};

use crate::{
    attributes, Axis, BoxConstraints, Children, DrawContext, Event, EventContext, EventSignal,
    Events, LayoutContext, Length, Node, Parent, PointerEvent, Properties, Scope, StyleClass,
    StyleClasses, TransitionState, View,
};

#[derive(Default)]
pub struct Div {
    pub classes: StyleClasses,
    pub direction: Option<Axis>,
    pub padding: Option<Length>,
    pub gap: Option<Length>,
    pub background: Option<Color>,
    pub border_radius: Option<Length>,
    pub border_width: Option<Length>,
    pub border_color: Option<Color>,
    pub on_press: Option<EventSignal<PointerEvent>>,
    pub on_release: Option<EventSignal<PointerEvent>>,
    pub children: Children,
}

impl Div {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn zeroed() -> Self {
        Self {
            direction: Some(Axis::Vertical),
            padding: Some(Length::ZERO),
            gap: Some(Length::ZERO),
            background: Some(Color::TRANSPARENT),
            border_radius: Some(Length::ZERO),
            border_width: Some(Length::ZERO),
            border_color: Some(Color::TRANSPARENT),
            ..Default::default()
        }
    }

    pub fn child(mut self, child: impl View) -> Self {
        self.add_child(child);
        self
    }

    pub fn class(mut self, class: impl Into<StyleClass>) -> Self {
        self.classes.push(class);
        self
    }

    pub fn on_press<'a>(mut self, cx: Scope<'a>, callback: impl FnMut(&PointerEvent) + 'a) -> Self {
        self.on_press
            .get_or_insert_with(|| EventSignal::new())
            .subscribe(cx, callback);

        self
    }

    pub fn on_release<'a>(
        mut self,
        cx: Scope<'a>,
        callback: impl FnMut(&PointerEvent) + 'a,
    ) -> Self {
        self.on_release
            .get_or_insert_with(|| EventSignal::new())
            .subscribe(cx, callback);

        self
    }

    pub fn direction(mut self, direction: Axis) -> Self {
        self.direction = Some(direction);
        self
    }

    pub fn padding(mut self, padding: impl Into<Length>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    pub fn gap(mut self, gap: impl Into<Length>) -> Self {
        self.gap = Some(gap.into());
        self
    }

    pub fn background(mut self, background: Color) -> Self {
        self.background = Some(background);
        self
    }

    pub fn border_radius(mut self, border_radius: impl Into<Length>) -> Self {
        self.border_radius = Some(border_radius.into());
        self
    }

    pub fn border_width(mut self, border_width: impl Into<Length>) -> Self {
        self.border_width = Some(border_width.into());
        self
    }

    pub fn border_color(mut self, border_color: Color) -> Self {
        self.border_color = Some(border_color);
        self
    }

    fn handle_pointer_event(&self, cx: &mut EventContext, event: &PointerEvent) -> bool {
        if event.button.is_some() && event.pressed && cx.hovered() {
            if let Some(on_press) = &self.on_press {
                cx.state.active = true;
                on_press.emit(event.clone());
                cx.request_redraw();
            }
        } else if !event.pressed && cx.state.active {
            cx.state.active = false;
            cx.request_redraw();

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

pub struct DivProperties<'a> {
    div: &'a mut Div,
}

impl<'a> DivProperties<'a> {
    pub fn class(&mut self, class: impl Into<StyleClass>) {
        self.div.classes.push(class);
    }

    pub fn direction(&mut self, direction: Axis) {
        self.div.direction = Some(direction);
    }

    pub fn padding(&mut self, padding: impl Into<Length>) {
        self.div.padding = Some(padding.into());
    }

    pub fn gap(&mut self, gap: impl Into<Length>) {
        self.div.gap = Some(gap.into());
    }

    pub fn background(&mut self, background: Color) {
        self.div.background = Some(background);
    }

    pub fn border_radius(&mut self, border_radius: impl Into<Length>) {
        self.div.border_radius = Some(border_radius.into());
    }

    pub fn border_width(&mut self, border_width: impl Into<Length>) {
        self.div.border_width = Some(border_width.into());
    }

    pub fn border_color(&mut self, border_color: Color) {
        self.div.border_color = Some(border_color);
    }
}

impl Properties for Div {
    type Setter<'a> = DivProperties<'a>;

    fn setter(&mut self) -> Self::Setter<'_> {
        Self::Setter { div: self }
    }
}

pub struct DivEvents<'a> {
    div: &'a mut Div,
}

impl<'a> DivEvents<'a> {
    pub fn press<'b>(
        &mut self,
        cx: Scope<'b>,
        callback: impl FnMut(&PointerEvent) + 'b,
    ) -> &mut Self {
        self.div
            .on_press
            .get_or_insert_with(|| EventSignal::new())
            .subscribe(cx, callback);

        self
    }

    pub fn release<'b>(
        &mut self,
        cx: Scope<'b>,
        callback: impl FnMut(&PointerEvent) + 'b,
    ) -> &mut Self {
        self.div
            .on_release
            .get_or_insert_with(|| EventSignal::new())
            .subscribe(cx, callback);

        self
    }
}

impl Events for Div {
    type Setter<'a> = DivEvents<'a>;

    fn setter(&mut self) -> Self::Setter<'_> {
        Self::Setter { div: self }
    }
}

#[derive(Default)]
pub struct DivState {
    padding: TransitionState<Length>,
    gap: TransitionState<Length>,
    background: TransitionState<Color>,
    border_radius: TransitionState<Length>,
    border_width: TransitionState<Length>,
    border_color: TransitionState<Color>,
}

impl View for Div {
    type State = DivState;

    fn build(&self) -> Self::State {
        DivState::default()
    }

    fn element(&self) -> Option<&'static str> {
        Some("div")
    }

    fn classes(&self) -> StyleClasses {
        self.classes.clone()
    }

    fn event(&self, _state: &mut Self::State, cx: &mut EventContext, event: &Event) {
        for child in &self.children {
            child.event(cx, event);
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

    fn layout(&self, state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        attributes! {
            cx, self,
            direction: "direction",
            padding: "padding" (state.padding),
            gap: "gap" (state.gap),
        }

        let padding = padding.pixels();
        let gap = gap.pixels();

        let mut major = padding;
        let mut minor = direction.minor(bc.min);

        let min_minor = direction.minor(bc.min) - padding * 2.0;
        let max_minor = direction.minor(bc.max) - padding * 2.0;

        for (i, child) in self.children.iter().enumerate() {
            let child_bc = BoxConstraints {
                min: direction.pack(0.0, min_minor),
                max: direction.pack(f32::INFINITY, max_minor),
            };

            let child_size = child.layout(cx, child_bc);
            let child_major = direction.major(child_size);
            child.set_offset(direction.pack(major, padding));

            // skip children that are too small
            if child_size.min_element() <= 0.0 {
                continue;
            }

            major += child_major;
            minor = minor.max(direction.minor(child_size + padding * 2.0));

            if i < self.children.len() - 1 {
                major += gap;
            }
        }

        major += padding;
        major = major.max(direction.major(bc.min));

        tracing::trace!("Div::layout: major = {}, minor = {}", major, minor);

        direction.pack(major, minor)
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        tracing::trace!("Div::draw: rect = {:?}", cx.rect());

        attributes! {
            cx, self,
            background: "background" (state.background),
            border_radius: "border-radius" (state.border_radius),
            border_width: "border-width" (state.border_width),
            border_color: "border-color" (state.border_color),
        }

        let border_radius = border_radius.pixels();
        let border_width = border_width.pixels();

        let quad = Quad {
            rect: cx.rect(),
            background,
            border_radius: [border_radius; 4],
            border_width,
            border_color,
        };

        cx.draw_primitive(quad);

        cx.layer(|cx| {
            for child in &self.children {
                child.draw(cx);
            }
        });
    }
}
