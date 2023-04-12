use glam::Vec2;
use ily_graphics::{Color, Quad};

use crate::{
    BoxConstraints, DrawContext, Event, EventContext, EventSignal, Events, LayoutContext, Node,
    Parent, PointerEvent, Properties, Scope, View,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    pub const fn cross(self) -> Self {
        match self {
            Axis::Horizontal => Axis::Vertical,
            Axis::Vertical => Axis::Horizontal,
        }
    }

    pub const fn minor(self, size: Vec2) -> f32 {
        match self {
            Axis::Horizontal => size.y,
            Axis::Vertical => size.x,
        }
    }

    pub const fn major(self, size: Vec2) -> f32 {
        match self {
            Axis::Horizontal => size.x,
            Axis::Vertical => size.y,
        }
    }

    pub const fn pack(self, major: f32, minor: f32) -> Vec2 {
        match self {
            Axis::Horizontal => Vec2::new(major, minor),
            Axis::Vertical => Vec2::new(minor, major),
        }
    }
}

pub struct Div {
    pub direction: Axis,
    pub padding: f32,
    pub gap: f32,
    pub background: Color,
    pub background_hover: Option<Color>,
    pub border_radius: [f32; 4],
    pub border_width: f32,
    pub border_color: Color,
    pub on_press: Option<EventSignal<PointerEvent>>,
    pub on_release: Option<EventSignal<PointerEvent>>,
    pub children: Vec<Node>,
}

impl Default for Div {
    fn default() -> Self {
        Self {
            direction: Axis::Vertical,
            padding: 10.0,
            gap: 10.0,
            background: Default::default(),
            background_hover: Default::default(),
            border_radius: Default::default(),
            border_width: Default::default(),
            border_color: Default::default(),
            on_press: Default::default(),
            on_release: Default::default(),
            children: Default::default(),
        }
    }
}

impl Div {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn zeroed() -> Self {
        Self {
            direction: Axis::Vertical,
            padding: 0.0,
            gap: 0.0,
            background: Color::TRANSPARENT,
            ..Default::default()
        }
    }

    pub fn child(mut self, child: impl View) -> Self {
        self.add_child(child);
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
        self.direction = direction;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn background(mut self, background: Color) -> Self {
        self.background = background;
        self
    }

    pub fn background_hover(mut self, background_hover: impl Into<Option<Color>>) -> Self {
        self.background_hover = background_hover.into();
        self
    }

    pub fn border_radius(mut self, border_radius: f32) -> Self {
        self.border_radius = [border_radius; 4];
        self
    }

    pub fn border_width(mut self, border_width: f32) -> Self {
        self.border_width = border_width;
        self
    }

    pub fn border_color(mut self, border_color: Color) -> Self {
        self.border_color = border_color;
        self
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
    pub fn direction(&mut self, direction: Axis) {
        self.div.direction = direction;
    }

    pub fn padding(&mut self, padding: f32) {
        self.div.padding = padding;
    }

    pub fn gap(&mut self, gap: f32) {
        self.div.gap = gap;
    }

    pub fn background(&mut self, background: Color) {
        self.div.background = background;
    }

    pub fn background_hover(&mut self, background_hover: impl Into<Option<Color>>) {
        self.div.background_hover = background_hover.into();
    }

    pub fn border_radius(&mut self, border_radius: f32) {
        self.div.border_radius = [border_radius; 4];
    }

    pub fn border_width(&mut self, border_width: f32) {
        self.div.border_width = border_width;
    }

    pub fn border_color(&mut self, border_color: Color) {
        self.div.border_color = border_color;
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

impl View for Div {
    type State = ();

    fn build(&self) -> Self::State {}

    fn element(&self) -> Option<&'static str> {
        Some("div")
    }

    fn event(&self, _state: &mut Self::State, cx: &mut EventContext, event: &Event) {
        for child in &self.children {
            child.event(cx, event);
        }

        if event.is_handled() {
            return;
        }

        if let Some(pointer_event) = event.get::<PointerEvent>() {
            if pointer_event.pressed {
                if let Some(on_press) = &self.on_press {
                    on_press.emit(pointer_event.clone());
                    event.handle();
                }
            } else {
                if let Some(on_release) = &self.on_release {
                    on_release.emit(pointer_event.clone());
                    event.handle();
                }
            }
        }
    }

    fn layout(&self, _state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        let mut major = self.padding;
        let mut minor = self.direction.minor(bc.min);

        let max_minor = self.direction.minor(bc.max) - self.padding * 2.0;

        for (i, child) in self.children.iter().enumerate() {
            let child_bc = BoxConstraints {
                min: self.direction.pack(0.0, minor),
                max: self.direction.pack(f32::INFINITY, max_minor),
            };

            let child_size = child.layout(cx, child_bc);
            let child_major = self.direction.major(child_size);
            child.set_offset(self.direction.pack(major, self.padding));

            major += child_major;
            minor = minor.max(self.direction.minor(child_size + self.padding * 2.0));

            if i < self.children.len() - 1 {
                major += self.gap;
            }
        }

        major += self.padding;
        major = major.max(self.direction.major(bc.min));

        tracing::trace!("Div::layout: major = {}, minor = {}", major, minor);

        self.direction.pack(major, minor)
    }

    fn draw(&self, _state: &mut Self::State, cx: &mut DrawContext) {
        tracing::trace!("Div::draw: rect = {:?}", cx.rect());

        let quad = Quad {
            rect: cx.rect(),
            background: if cx.hovered() {
                self.background_hover.unwrap_or(self.background)
            } else {
                self.background
            },
            border_radius: self.border_radius,
            border_width: self.border_width,
            border_color: self.border_color,
        };

        cx.draw_primitive(quad);

        cx.layer(|cx| {
            for child in &self.children {
                child.draw(cx);
            }
        });
    }
}
