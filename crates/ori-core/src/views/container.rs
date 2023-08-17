use crate::{
    builtin::container, style, Align, BuildCx, Canvas, Color, DrawCx, Event, EventCx, LayoutCx,
    Padding, Pod, PodState, Rebuild, RebuildCx, Size, Space, View,
};

#[derive(Rebuild)]
pub struct Container<T, V> {
    pub content: Pod<T, V>,
    #[rebuild(layout)]
    pub padding: Padding,
    #[rebuild(layout)]
    pub size: Option<Size>,
    #[rebuild(layout)]
    pub alignment: Option<Align>,
    #[rebuild(draw)]
    pub background: Color,
    #[rebuild(draw)]
    pub border_radius: [f32; 4],
    #[rebuild(draw)]
    pub border_width: [f32; 4],
    #[rebuild(draw)]
    pub border_color: Color,
}

impl<T, V> Container<T, V> {
    pub fn new(content: V) -> Self {
        Self {
            content: Pod::new(content),
            padding: Padding::default(),
            size: None,
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
        self.size = Some(size.into());
        self
    }

    pub fn align(mut self, alignment: impl Into<Align>) -> Self {
        self.alignment = Some(alignment.into());
        self
    }

    pub fn background(mut self, background: impl Into<Color>) -> Self {
        self.background = background.into();
        self
    }

    pub fn border_radius(mut self, border_radius: impl Into<[f32; 4]>) -> Self {
        self.border_radius = border_radius.into();
        self
    }

    pub fn border_width(mut self, border_width: impl Into<[f32; 4]>) -> Self {
        self.border_width = border_width.into();
        self
    }

    pub fn border_color(mut self, border_color: impl Into<Color>) -> Self {
        self.border_color = border_color.into();
        self
    }
}

impl<T, V: View<T>> View<T> for Container<T, V> {
    type State = PodState<T, V>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        self.content.build(cx, data)
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);

        self.content.rebuild(state, cx, data, &old.content);
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        self.content.event(state, cx, data, event);
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        let content_space = space.shrink(self.padding.size());
        let mut content_size = self.content.layout(state, cx, data, content_space);
        content_size += self.padding.size();

        if let Some(alignment) = self.alignment {
            let size = self.size.unwrap_or(Size::UNBOUNDED);
            let size = space.fit_container(content_size, size);
            let align = alignment.align(content_size, size);
            state.translate(self.padding.offset() + align);

            return size;
        }

        state.translate(self.padding.offset());

        space.fit_container(content_size, self.size.unwrap_or(Size::ZERO))
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        canvas.draw_quad(
            cx.rect(),
            self.background,
            self.border_radius,
            self.border_width,
            self.border_color,
        );

        self.content.draw(state, cx, data, canvas);
    }
}

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
        size: Some(size.into()),
        ..Container::new(content)
    }
}

pub fn align<T, V: View<T>>(alignment: impl Into<Align>, content: V) -> Container<T, V> {
    Container {
        alignment: Some(alignment.into()),
        ..Container::new(content)
    }
}

pub fn align_center<T, V: View<T>>(content: V) -> Container<T, V> {
    Container {
        alignment: Some(Align::CENTER),
        ..Container::new(content)
    }
}

pub fn align_top_left<T, V: View<T>>(content: V) -> Container<T, V> {
    Container {
        alignment: Some(Align::TOP_LEFT),
        ..Container::new(content)
    }
}

pub fn align_top<T, V: View<T>>(content: V) -> Container<T, V> {
    Container {
        alignment: Some(Align::TOP),
        ..Container::new(content)
    }
}

pub fn align_top_right<T, V: View<T>>(content: V) -> Container<T, V> {
    Container {
        alignment: Some(Align::TOP_RIGHT),
        ..Container::new(content)
    }
}

pub fn align_left<T, V: View<T>>(content: V) -> Container<T, V> {
    Container {
        alignment: Some(Align::LEFT),
        ..Container::new(content)
    }
}

pub fn align_right<T, V: View<T>>(content: V) -> Container<T, V> {
    Container {
        alignment: Some(Align::RIGHT),
        ..Container::new(content)
    }
}

pub fn align_bottom_left<T, V: View<T>>(content: V) -> Container<T, V> {
    Container {
        alignment: Some(Align::BOTTOM_LEFT),
        ..Container::new(content)
    }
}

pub fn align_bottom<T, V: View<T>>(content: V) -> Container<T, V> {
    Container {
        alignment: Some(Align::BOTTOM),
        ..Container::new(content)
    }
}

pub fn align_bottom_right<T, V: View<T>>(content: V) -> Container<T, V> {
    Container {
        alignment: Some(Align::BOTTOM_RIGHT),
        ..Container::new(content)
    }
}
