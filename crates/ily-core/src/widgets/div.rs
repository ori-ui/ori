use glam::Vec2;
use ily_graphics::{Color, Quad, Rect};
use ily_reactive::{BoundedScope, Scope};

use crate::{AnyView, BoxConstraints, Event, PaintContext, View};

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

    fn constraints(self, bc: &BoxConstraints, min_major: f32, major: f32) -> BoxConstraints {
        match self {
            Axis::Horizontal => BoxConstraints {
                min: Vec2::new(min_major, bc.min.y),
                max: Vec2::new(major, bc.max.y),
            },
            Axis::Vertical => BoxConstraints {
                min: Vec2::new(bc.min.x, min_major),
                max: Vec2::new(bc.max.x, major),
            },
        }
    }
}

pub struct Div {
    direction: Axis,
    padding: f32,
    gap: f32,
    background: Color,
    border_radius: [f32; 4],
    border_width: f32,
    border_color: Color,
    children: Vec<AnyView>,
}

impl Div {
    pub fn new() -> Self {
        tracing::trace!("creating Div");

        Self {
            direction: Axis::Vertical,
            padding: 0.0,
            gap: 0.0,
            background: Color::WHITE,
            border_radius: [0.0; 4],
            border_width: 5.0,
            border_color: Color::BLACK,
            children: Vec::new(),
        }
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

    pub fn child<'a, V: View>(
        mut self,
        cx: Scope<'a>,
        f: impl FnMut(BoundedScope<'_, 'a>) -> V + 'a,
    ) -> Self {
        self.children.push(AnyView::new(cx, f));
        self
    }
}

impl View for Div {
    fn classes(&self) -> Vec<String> {
        Vec::new()
    }

    fn event(&self, event: &Event) {
        for child in &self.children {
            child.event(event);
        }
    }

    fn layout(&self, bc: BoxConstraints) -> Vec2 {
        let loose = bc.loose();

        let mut major = self.padding;
        let mut minor = self.direction.minor(bc.min);

        for child in &self.children {
            let child_bc = self.direction.constraints(&loose, minor, f32::INFINITY);
            let child_size = child.layout(child_bc);

            let min = self.direction.pack(major, self.padding);
            let child_rect = Rect::min_size(min, child_size);
            child.set_rect(child_rect);

            major += self.direction.major(child_size);
            minor = self
                .direction
                .minor(child_size + self.padding * 2.0)
                .max(minor);
        }

        major += self.padding;
        major = major.max(self.direction.major(bc.min));

        tracing::trace!("Div::layout: major = {}, minor = {}", major, minor);

        self.direction.pack(major, minor)
    }

    fn paint(&self, cx: &mut PaintContext) {
        let quad = Quad {
            rect: cx.rect(),
            background: self.background,
            border_radius: self.border_radius,
            border_width: self.border_width,
            border_color: self.border_color,
        };

        tracing::trace!("Div::paint: quad = {:#?}", quad);
        cx.draw_primitive(quad);

        for child in &self.children {
            let child_rect = child.rect();
            tracing::trace!("Div::paint: child_rect = {:?}", child_rect);
            cx.layer(|cx| child.paint(cx));
        }
    }
}
