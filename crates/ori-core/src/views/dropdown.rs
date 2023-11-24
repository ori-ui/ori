use ori_macro::Rebuild;

use crate::{
    canvas::Canvas,
    event::{Event, PointerPressed},
    layout::{Size, Space, Vector},
    view::{BuildCx, DrawCx, EventCx, LayoutCx, Pod, RebuildCx, State, View},
};

/// Create a new [`Dropdown`] view.
pub fn dropdown<H, V>(header: H, content: V) -> Dropdown<H, V> {
    Dropdown::new(header, content)
}

/// A dropdown view.
#[derive(Rebuild)]
pub struct Dropdown<H, V> {
    /// Whether the dropdown is toggled.
    #[rebuild(layout)]
    pub toggle: bool,
    /// The header of the dropdown.
    pub header: H,
    /// The content of the dropdown.
    pub content: Pod<V>,
}

impl<H, V> Dropdown<H, V> {
    /// Create a new dropdown.
    pub fn new(header: H, content: V) -> Self {
        Self {
            toggle: false,
            header,
            content: Pod::new(content),
        }
    }

    /// Set whether the dropdown is toggled.
    pub fn toggle(mut self, toggle: bool) -> Self {
        self.toggle = toggle;
        self
    }
}

impl<T, H: View<T>, V: View<T>> View<T> for Dropdown<H, V> {
    type State = (H::State, State<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        (self.header.build(cx, data), self.content.build(cx, data))
    }

    fn rebuild(
        &mut self,
        (header, content): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        self.header.rebuild(header, cx, data, &old.header);
        self.content.rebuild(content, cx, data, &old.content);
    }

    fn event(
        &mut self,
        (header, content): &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) {
        self.header.event(header, cx, data, event);

        if cx.is_focused() {
            self.content.event(content, cx, data, event);
        }

        if !self.toggle && !cx.is_focused() && cx.is_hot() {
            cx.set_focused(true);
            cx.request_draw();
        }

        if event.is::<PointerPressed>() {
            if !self.toggle && cx.is_focused() && !cx.has_hot() {
                cx.set_focused(false);
                cx.request_draw();
            }

            if self.toggle && (cx.is_hot() || cx.is_focused()) && !content.has_hot() {
                cx.set_focused(!cx.is_focused());
                cx.request_draw();
            }
        }
    }

    fn layout(
        &mut self,
        (header, content): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        let mut header_size = self.header.layout(header, cx, data, space);

        // make sure the content is at least as wide as the header
        let content_space = Space {
            min: Size::new(header_size.width, 0.0),
            max: cx.window().size(),
        };
        let content_size = self.content.layout(content, cx, data, content_space);

        // if the content is wider than the header, make the header as wide as the content
        if content_size.width > header_size.width {
            let header_space = Space {
                min: Size::new(content_size.width, space.min.height),
                max: Size::new(content_size.width, space.max.height),
            };

            header_size = self.header.layout(header, cx, data, header_space);
        }

        // translate the content below the header
        content.translate(Vector::new(0.0, header_size.height));

        header_size
    }

    fn draw(
        &mut self,
        (header, content): &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        canvas.view(cx.id());
        canvas.trigger(cx.rect());

        self.header.draw(header, cx, data, canvas);

        if !cx.is_focused() {
            return;
        }

        let mut layer = canvas.layer();
        layer.depth += 10000.0;
        self.content.draw(content, cx, data, &mut layer);
    }
}
