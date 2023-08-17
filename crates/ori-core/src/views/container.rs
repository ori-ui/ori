use crate::{
    container, style, Align, Canvas, Color, DrawCx, Event, EventCx, LayoutCx, Padding, Pod,
    PodState, RebuildCx, Size, Space, View,
};

pub fn container<T, V: View<T>>(content: V) -> Container<T, V> {
    Container::new(content)
}

pub fn pad<T, V: View<T>>(padding: impl Into<Padding>, content: V) -> Container<T, V> {
    Container {
        padding: padding.into(),
        ..Container::new(content)
    }
}

pub fn size<T, V: View<T>>(size: impl Into<Size>, content: V) -> Container<T, V> {
    Container {
        size: size.into(),
        ..Container::new(content)
    }
}

pub fn align<T, V: View<T>>(alignment: impl Into<Align>, content: V) -> Container<T, V> {
    Container {
        alignment: Some(alignment.into()),
        ..Container::new(content)
    }
}

pub struct Container<T, V> {
    pub content: Pod<T, V>,
    pub padding: Padding,
    pub size: Size,
    pub alignment: Option<Align>,
    pub background: Color,
    pub border_radius: [f32; 4],
    pub border_width: [f32; 4],
    pub border_color: Color,
}

impl<T, V> Container<T, V> {
    pub fn new(content: V) -> Self {
        Self {
            content: Pod::new(content),
            padding: Padding::default(),
            size: Size::ZERO,
            alignment: None,
            background: style(container::BACKGROUND),
            border_radius: style(container::BORDER_RADIUS),
            border_width: style(container::BORDER_WIDTH),
            border_color: style(container::BORDER_COLOR),
        }
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }

    pub fn align(mut self, alignment: impl Into<Align>) -> Self {
        self.alignment = Some(alignment.into());
        self
    }
}

impl<T, V: View<T>> View<T> for Container<T, V> {
    type State = PodState<T, V>;

    fn build(&self) -> Self::State {
        self.content.build()
    }

    fn rebuild(&mut self, cx: &mut RebuildCx, old: &Self, state: &mut Self::State) {
        if self.padding != old.padding {
            cx.request_draw();
        }

        self.content.rebuild(cx, &old.content, state);
    }

    fn event(&mut self, cx: &mut EventCx, state: &mut Self::State, data: &mut T, event: &Event) {
        self.content.event(cx, state, data, event);
    }

    fn layout(&mut self, cx: &mut LayoutCx, state: &mut Self::State, space: Space) -> Size {
        let content_space = space.shrink(self.padding.size());
        let mut content_size = self.content.layout(cx, state, content_space);
        content_size += self.padding.size();

        if let Some(alignment) = self.alignment {
            let size = space.fit_container(content_size, Size::UNBOUNDED);
            let align = alignment.align(content_size, size);
            state.translate(self.padding.offset() + align);

            return size;
        }

        state.translate(self.padding.offset());

        space.fit_container(content_size, self.size)
    }

    fn draw(&mut self, cx: &mut DrawCx, state: &mut Self::State, canvas: &mut Canvas) {
        canvas.draw_quad(
            cx.rect(),
            self.background,
            self.border_radius,
            self.border_width,
            self.border_color,
        );

        self.content.draw(cx, state, canvas);
    }
}
