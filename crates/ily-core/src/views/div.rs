use glam::Vec2;
use ily_macro::Build;

use crate::{
    AlignItems, Axis, BoxConstraints, Children, Context, DrawContext, Event, EventContext,
    EventSignal, JustifyContent, LayoutContext, Node, Parent, PointerEvent, Scope, Style, View,
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

    pub fn on_event<'a>(mut self, cx: Scope<'a>, callback: impl FnMut(&Event) + 'a) -> Self {
        self.on_event
            .get_or_insert_with(|| EventSignal::new())
            .subscribe(cx, callback);

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
        let axis = cx.style::<Axis>("direction");
        let justify_content = cx.style::<JustifyContent>("justify-content");
        let align_items = cx.style::<AlignItems>("align-items");

        let min_width = cx.style_range_or("width", "min-width", bc.width());
        let max_width = cx.style_range_or("width", "max-width", bc.width());

        let min_height = cx.style_range_or("height", "min-height", bc.height());
        let max_height = cx.style_range_or("height", "max-height", bc.height());

        let padding = cx.style_range("padding", 0.0..bc.max.min_element() / 2.0);
        let gap = cx.style_range("gap", 0.0..axis.major(bc.max));

        let min_size = bc.constrain(Vec2::new(min_width, min_height));
        let max_size = bc.constrain(Vec2::new(max_width, max_height));

        let max_minor = axis.minor(max_size) - padding * 2.0;
        let min_minor = axis.minor(min_size) - padding * 2.0;

        let max_major = axis.major(max_size) - padding * 2.0;
        let min_major = axis.major(min_size) - padding * 2.0;

        let mut minor = min_minor;

        // first we need to measure the children to determine their size
        for child in self.children.iter() {
            let child_bc = BoxConstraints {
                min: axis.pack(0.0, 0.0),
                max: axis.pack(max_major, max_minor),
            };
            let size = child.layout(cx, child_bc);
            let child_minor = axis.minor(size);

            minor = minor.max(child_minor);
        }

        let mut major = 0.0f32;

        let min_minor = if align_items == AlignItems::Stretch {
            minor
        } else {
            0.0
        };

        let mut children = Vec::new();
        for (i, child) in self.children.iter().enumerate() {
            let child_bc = BoxConstraints {
                min: axis.pack(0.0, min_minor),
                max: axis.pack(max_major, max_minor),
            };
            let size = child.layout(cx, child_bc);
            let child_major = axis.major(size);

            children.push(child_major);

            major += child_major;

            if i > 0 {
                major += gap;
            }
        }

        major = major.max(min_major);

        let child_offsets = justify_content.justify(&children, major, gap);

        // now we can layout the children
        for (i, align_major) in child_offsets.into_iter().enumerate() {
            let child = &self.children[i];

            let child_minor = axis.minor(child.size());

            let align_minor = align_items.align(0.0, minor, child_minor);

            let offset = axis.pack(align_major, align_minor);
            child.set_offset(offset + padding);
        }

        let size = axis.pack(major, minor) + padding * 2.0;

        size
    }

    fn draw(&self, _state: &mut Self::State, cx: &mut DrawContext) {
        cx.draw_quad();

        cx.layer(|cx| {
            for child in &self.children {
                child.draw(cx);
            }
        });
    }
}
