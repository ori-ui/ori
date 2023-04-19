use glam::Vec2;
use ily_macro::Build;
use smallvec::SmallVec;

use crate::{
    AlignItems, Axis, BoxConstraints, Children, Context, DrawContext, Event, EventContext,
    EventSignal, JustifyContent, LayoutContext, Node, Parent, PointerEvent, Scope, Sendable, Style,
    View,
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

    pub fn on_event<'a>(
        mut self,
        cx: Scope<'a>,
        callback: impl FnMut(&Event) + Sendable + 'a,
    ) -> Self {
        self.on_event
            .get_or_insert_with(|| EventSignal::new())
            .subscribe(cx, callback);

        self
    }

    pub fn on_press<'a>(
        mut self,
        cx: Scope<'a>,
        callback: impl FnMut(&PointerEvent) + Sendable + 'a,
    ) -> Self {
        self.on_press
            .get_or_insert_with(|| EventSignal::new())
            .subscribe(cx, callback);

        self
    }

    pub fn on_release<'a>(
        mut self,
        cx: Scope<'a>,
        callback: impl FnMut(&PointerEvent) + Sendable + 'a,
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

        let bc = cx.style_constraints(bc);

        let padding = cx.style_range("padding", 0.0..bc.max.min_element() / 2.0);
        let gap = cx.style_range("gap", 0.0..axis.major(bc.max));

        let max_minor = axis.minor(bc.max) - padding * 2.0;
        let min_minor = axis.minor(bc.min) - padding * 2.0;

        let max_major = axis.major(bc.max) - padding * 2.0;
        let min_major = axis.major(bc.min) - padding * 2.0;

        let mut minor = min_minor;
        let mut major = 0.0f32;

        // first we need to measure the children to determine their size
        //
        // NOTE: using a SmallVec here is a bit faster than using a Vec, but it's not a huge
        // difference
        let mut children = SmallVec::<[f32; 8]>::with_capacity(self.children.len());
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

        if align_items == AlignItems::Stretch {
            // we need to re-measure the children to determine their size
            children.clear();

            for child in self.children.iter() {
                let child_bc = BoxConstraints {
                    min: axis.pack(0.0, minor),
                    max: axis.pack(max_major, minor),
                };
                // FIXME: calling layout again is not ideal, but it's the only way to get the
                // correct size for the child, since we don't know the minor size until we've
                // measured all the children
                let size = child.layout(cx, child_bc);
                let child_major = axis.major(size);

                children.push(child_major);
            }
        }

        major = major.max(min_major);

        let child_offsets = justify_content.justify(&children, major, gap);

        // now we can layout the children
        for (child, align_major) in self.children.iter().zip(child_offsets) {
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
