use glam::Vec2;
use ily_graphics::Quad;

use crate::{
    AlignItems, Axis, BoxConstraints, Children, DrawContext, Event, EventContext, EventSignal,
    Events, JustifyContent, LayoutContext, Node, Parent, PointerEvent, Properties, Scope,
    StyleClass, StyleClasses, View,
};

#[derive(Default)]
pub struct Div {
    pub classes: StyleClasses,
    pub on_press: Option<EventSignal<PointerEvent>>,
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

    fn layout(&self, _state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        let axis = cx.style::<Axis>("direction");
        let justify_content = cx.style::<JustifyContent>("justify-content");
        let align_items = cx.style::<AlignItems>("align-items");

        let min_width = cx.style_range("min-width", bc.width());
        let max_width = cx.style_range("max-width", bc.width());

        let min_height = cx.style_range("min-height", bc.height());
        let max_height = cx.style_range("max-height", bc.height());

        let padding = cx.style_range("padding", 0.0..bc.max.min_element() / 2.0);
        let gap = cx.style_range("gap", 0.0..axis.major(bc.max));

        let min_size = bc.constrain(Vec2::new(min_width, min_height));
        let max_size = bc.constrain(Vec2::new(max_width, max_height));

        let max_minor = axis.minor(max_size) - padding * 2.0;
        let min_minor = axis.minor(min_size) - padding * 2.0;

        let max_major = axis.major(max_size) - padding * 2.0;
        let min_major = axis.major(min_size) - padding * 2.0;

        let mut minor = min_minor;
        let mut major = 0.0f32;

        let mut children = Vec::with_capacity(self.children.len());

        // first we need to measure the children to determine their size
        for (i, child) in self.children.iter().enumerate() {
            let child_bc = BoxConstraints {
                min: axis.pack(0.0, 0.0),
                max: axis.pack(max_major, max_minor),
            };
            let size = child.layout(cx, child_bc);

            let child_minor = axis.minor(size);
            let child_major = axis.major(size);

            children.push(child_major);

            minor = minor.max(child_minor);
            major += child_major;

            if i > 0 {
                major += gap;
            }
        }

        major = major.clamp(min_major, max_major);

        tracing::trace!("Div::layout: minor = {}, major = {}", minor, major);

        let child_offsets = justify_content.justify(&children, major, gap);

        // now we can layout the children
        for (i, child) in self.children.iter().enumerate() {
            let min_minor = if align_items == AlignItems::Stretch {
                minor
            } else {
                0.0
            };

            let child_bc = BoxConstraints {
                min: axis.pack(0.0, min_minor),
                max: axis.pack(max_major, max_minor),
            };
            let child_size = child.layout(cx, child_bc);

            let child_minor = axis.minor(child_size);

            let align_minor = align_items.align(0.0, minor, child_minor);
            let align_major = child_offsets[i];

            let offset = axis.pack(align_major, align_minor);
            child.set_offset(offset + padding);
        }

        let size = axis.pack(major, minor) + padding * 2.0;
        tracing::trace!("Div::layout: size = {:?}", size);

        size
    }

    fn draw(&self, _state: &mut Self::State, cx: &mut DrawContext) {
        tracing::trace!("Div::draw: rect = {:?}", cx.rect());

        let range = 0.0..cx.rect().max.min_element() / 2.0;
        let border_radius = cx.style_range("border-radius", range.clone());
        let border_width = cx.style_range("border-width", range);

        let background = cx.style("background");
        let border_color = cx.style("border-color");

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
