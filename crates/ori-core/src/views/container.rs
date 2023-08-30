use glam::Vec2;

use crate::{
    canvas::{BorderRadius, BorderWidth, Canvas, Color},
    event::Event,
    layout::{Affine, Alignment, Padding, Size, Space},
    rebuild::Rebuild,
    theme::{container, style},
    view::{BuildCx, Content, DrawCx, EventCx, LayoutCx, RebuildCx, State, View},
};

/// A container view.
#[derive(Rebuild)]
pub struct Container<V> {
    /// The content.
    pub content: Content<V>,
    /// The padding, applied before everything else.
    #[rebuild(layout)]
    pub padding: Padding,
    /// The space available to the content.
    ///
    /// By default, the content is given [`Space::UNBOUNDED`].
    #[rebuild(layout)]
    pub space: Space,
    /// The alignment of the content.
    ///
    /// If set the container will try to fill the available space.
    #[rebuild(layout)]
    pub alignment: Option<Alignment>,
    /// The transform.
    ///
    /// This is applied after padding.
    #[rebuild(layout)]
    pub transform: Affine,
    /// The background color.
    #[rebuild(draw)]
    pub background: Color,
    /// The border radius.
    #[rebuild(draw)]
    pub border_radius: BorderRadius,
    /// The border width.
    #[rebuild(draw)]
    pub border_width: BorderWidth,
    /// The border color.
    #[rebuild(draw)]
    pub border_color: Color,
}

impl<V> Container<V> {
    /// Create a new [`Container`].
    pub fn new(content: V) -> Self {
        Self {
            content: Content::new(content),
            padding: Padding::default(),
            space: Space::default(),
            alignment: None,
            transform: Affine::IDENTITY,
            background: style(container::BACKGROUND),
            border_radius: style(container::BORDER_RADIUS),
            border_width: style(container::BORDER_WIDTH),
            border_color: style(container::BORDER_COLOR),
        }
    }

    /// Set the padding.    
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Set the size.
    pub fn size(mut self, size: impl Into<Size>) -> Self {
        self.space = Space::from_size(size.into());
        self
    }

    /// Set the width.
    pub fn width(mut self, width: f32) -> Self {
        self.space.min.width = width;
        self.space.max.width = width;
        self
    }

    /// Set the height.
    pub fn height(mut self, height: f32) -> Self {
        self.space.min.height = height;
        self.space.max.height = height;
        self
    }

    /// Set the minimum width.
    pub fn min_width(mut self, min_width: f32) -> Self {
        self.space.min.width = min_width;
        self
    }

    /// Set the minimum height.
    pub fn min_height(mut self, min_height: f32) -> Self {
        self.space.min.height = min_height;
        self
    }

    /// Set the maximum width.
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.space.max.width = max_width;
        self
    }

    /// Set the maximum height.
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.space.max.height = max_height;
        self
    }

    /// Set the alignment.
    pub fn align(mut self, alignment: impl Into<Alignment>) -> Self {
        self.alignment = Some(alignment.into());
        self
    }

    /// Set the transform.
    pub fn transform(mut self, transform: impl Into<Affine>) -> Self {
        self.transform = transform.into();
        self
    }

    /// Set the translation.
    pub fn translate(mut self, translation: impl Into<Vec2>) -> Self {
        self.transform = Affine::translate(translation.into());
        self
    }

    /// Set the rotation.
    pub fn rotate(mut self, rotation: f32) -> Self {
        self.transform = Affine::rotate(rotation);
        self
    }

    /// Set the scale.
    pub fn scale(mut self, scale: impl Into<Vec2>) -> Self {
        self.transform = Affine::scale(scale.into());
        self
    }

    /// Set the background color.
    pub fn background(mut self, background: impl Into<Color>) -> Self {
        self.background = background.into();
        self
    }

    /// Set the border radius.
    pub fn border_radius(mut self, border_radius: impl Into<BorderRadius>) -> Self {
        self.border_radius = border_radius.into();
        self
    }

    /// Set the border width.
    pub fn border_width(mut self, border_width: impl Into<BorderWidth>) -> Self {
        self.border_width = border_width.into();
        self
    }

    /// Set the border width of the top edge.
    pub fn border_top(mut self, width: f32) -> Self {
        self.border_width.top = width;
        self
    }

    /// Set the border width of the right edge.
    pub fn border_right(mut self, width: f32) -> Self {
        self.border_width.right = width;
        self
    }

    /// Set the border width of the bottom edge.
    pub fn border_bottom(mut self, width: f32) -> Self {
        self.border_width.bottom = width;
        self
    }

    /// Set the border width of the left edge.
    pub fn border_left(mut self, width: f32) -> Self {
        self.border_width.left = width;
        self
    }

    /// Set the border color.
    pub fn border_color(mut self, border_color: impl Into<Color>) -> Self {
        self.border_color = border_color.into();
        self
    }
}

impl<T, V: View<T>> View<T> for Container<V> {
    type State = State<T, V>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        cx.request_rebuild();
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
        let space = if self.alignment.is_some() {
            self.space.with(space).loosen()
        } else {
            self.space.with(space)
        };

        // the content must fit within the padding
        let content_space = space.shrink(self.padding.size());
        let mut content_size = self.content.layout(state, cx, data, content_space);
        content_size += self.padding.size();

        if let Some(alignment) = self.alignment {
            // try to fill the available space, this will be bounded by `self.space`
            let space = space.with(Space::FILL);
            let size = space.fit(content_size);

            // align the content within the self
            let align = alignment.align(content_size, size);

            // set the transform of the content
            let affine = Affine::translate(self.padding.offset() + align);
            state.set_transform(affine * self.transform);

            return size;
        }

        // set the transform of the content
        let affine = Affine::translate(self.padding.offset());
        state.set_transform(affine * self.transform);

        space.fit(content_size)
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

/// Create a new [`Container`].
pub fn container<V>(content: V) -> Container<V> {
    Container::new(content)
}

/// Create a new padded [`Container`].
pub fn pad<V>(padding: impl Into<Padding>, content: V) -> Container<V> {
    Container {
        padding: padding.into(),
        ..Container::new(content)
    }
}

/// Create a new [`Container`] with a fixed size.
pub fn size<V>(size: impl Into<Size>, content: V) -> Container<V> {
    Container {
        space: Space::from_size(size.into()),
        ..Container::new(content)
    }
}

/// Create a new [`Container`] with a fixed width.
pub fn width<V>(width: f32, content: V) -> Container<V> {
    Container::new(content).width(width)
}

/// Create a new [`Container`] with a fixed height.
pub fn height<V>(height: f32, content: V) -> Container<V> {
    Container::new(content).height(height)
}

/// Create a new [`Container`] with a minimum width.
pub fn min_width<V>(min_width: f32, content: V) -> Container<V> {
    Container::new(content).min_width(min_width)
}

/// Create a new [`Container`] with a minimum height.
pub fn min_height<V>(min_height: f32, content: V) -> Container<V> {
    Container::new(content).min_height(min_height)
}

/// Create a new [`Container`] with a maximum width.
pub fn max_width<V>(max_width: f32, content: V) -> Container<V> {
    Container::new(content).max_width(max_width)
}

/// Create a new [`Container`] with a maximum height.
pub fn max_height<V>(max_height: f32, content: V) -> Container<V> {
    Container::new(content).max_height(max_height)
}

/// Create a new aligned [`Container`].
pub fn align<V>(alignment: impl Into<Alignment>, content: V) -> Container<V> {
    Container {
        alignment: Some(alignment.into()),
        ..Container::new(content)
    }
}

/// Create a new transformed [`Container`].
pub fn transform<V>(transform: impl Into<Affine>, content: V) -> Container<V> {
    Container {
        transform: transform.into(),
        ..Container::new(content)
    }
}

/// Create a new translated [`Container`].
pub fn translate<V>(translation: impl Into<Vec2>, content: V) -> Container<V> {
    Container {
        transform: Affine::translate(translation.into()),
        ..Container::new(content)
    }
}

/// Create a new rotated [`Container`].
pub fn rotate<V>(rotation: f32, content: V) -> Container<V> {
    Container {
        transform: Affine::rotate(rotation),
        ..Container::new(content)
    }
}

/// Create a new scaled [`Container`].
pub fn scale<V>(scale: impl Into<Vec2>, content: V) -> Container<V> {
    Container {
        transform: Affine::scale(scale.into()),
        ..Container::new(content)
    }
}

/// Create a new [`Container`] aligned to the center.
pub fn center<V>(content: V) -> Container<V> {
    Container {
        alignment: Some(Alignment::CENTER),
        ..Container::new(content)
    }
}

/// Create a new [`Container`] aligned to the top left.
pub fn top_left<V>(content: V) -> Container<V> {
    Container {
        alignment: Some(Alignment::TOP_LEFT),
        ..Container::new(content)
    }
}

/// Create a new [`Container`] aligned to the top.
pub fn top<V>(content: V) -> Container<V> {
    Container {
        alignment: Some(Alignment::TOP),
        ..Container::new(content)
    }
}

/// Create a new [`Container`] aligned to the top right.
pub fn top_right<V>(content: V) -> Container<V> {
    Container {
        alignment: Some(Alignment::TOP_RIGHT),
        ..Container::new(content)
    }
}

/// Create a new [`Container`] aligned to the left.
pub fn left<V>(content: V) -> Container<V> {
    Container {
        alignment: Some(Alignment::LEFT),
        ..Container::new(content)
    }
}

/// Create a new [`Container`] aligned to the right.
pub fn right<V>(content: V) -> Container<V> {
    Container {
        alignment: Some(Alignment::RIGHT),
        ..Container::new(content)
    }
}

/// Create a new [`Container`] aligned to the bottom left.
pub fn bottom_left<V>(content: V) -> Container<V> {
    Container {
        alignment: Some(Alignment::BOTTOM_LEFT),
        ..Container::new(content)
    }
}

/// Create a new [`Container`] aligned to the bottom.
pub fn bottom<V>(content: V) -> Container<V> {
    Container {
        alignment: Some(Alignment::BOTTOM),
        ..Container::new(content)
    }
}

/// Create a new [`Container`] aligned to the bottom right.
pub fn bottom_right<V>(content: V) -> Container<V> {
    Container {
        alignment: Some(Alignment::BOTTOM_RIGHT),
        ..Container::new(content)
    }
}
