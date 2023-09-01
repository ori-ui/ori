use crate::{
    canvas::Canvas,
    event::Event,
    layout::{Size, Space},
    rebuild::Rebuild,
    view::{BuildCx, Content, DrawCx, EventCx, LayoutCx, RebuildCx, State, View},
};

/// Create a new [`Constrain`]ed view, constraining its content to a space.
pub fn constrain<V>(space: impl Into<Space>, content: V) -> Constrain<V> {
    Constrain::new(space.into(), content)
}

/// Create a new [`Constrain`]ed view, cosntraining its content to a size.
pub fn size<V>(size: impl Into<Size>, content: V) -> Constrain<V> {
    Constrain::new(Space::from_size(size.into()), content)
}

/// Create a new [`Constrain`]ed view, constraining its content to a minimum size.
pub fn min_size<V>(min_size: impl Into<Size>, content: V) -> Constrain<V> {
    Constrain::new(Space::new(min_size.into(), Size::FILL), content)
}

/// Create a new [`Constrain`]ed view, constraining its content to a maximum size.
pub fn max_size<V>(max_size: impl Into<Size>, content: V) -> Constrain<V> {
    Constrain::new(Space::new(Size::ZERO, max_size.into()), content)
}

/// Create a new [`Constrain`]ed view, constraining its content to a width.
pub fn width<V>(width: f32, content: V) -> Constrain<V> {
    let mut constrain = Constrain::unbounded(content);
    constrain.space.min.width = width;
    constrain.space.max.width = width;
    constrain
}

/// Create a new [`Constrain`]ed view, constraining its content to a height.
pub fn height<V>(height: f32, content: V) -> Constrain<V> {
    let mut constrain = Constrain::unbounded(content);
    constrain.space.min.height = height;
    constrain.space.max.height = height;
    constrain
}

/// Create a new [`Constrain`]ed view, constraining its content to a minimum width.
pub fn min_width<V>(min_width: f32, content: V) -> Constrain<V> {
    let mut constrain = Constrain::unbounded(content);
    constrain.space.min.width = min_width;
    constrain
}

/// Create a new [`Constrain`]ed view, constraining its content to a minimum height.
pub fn min_height<V>(min_height: f32, content: V) -> Constrain<V> {
    let mut constrain = Constrain::unbounded(content);
    constrain.space.min.height = min_height;
    constrain
}

/// Create a new [`Constrain`]ed view, constraining its content to a maximum width.
pub fn max_width<V>(max_width: f32, content: V) -> Constrain<V> {
    let mut constrain = Constrain::unbounded(content);
    constrain.space.max.width = max_width;
    constrain
}

/// Create a new [`Constrain`]ed view, constraining its content to a maximum height.
pub fn max_height<V>(max_height: f32, content: V) -> Constrain<V> {
    let mut constrain = Constrain::unbounded(content);
    constrain.space.max.height = max_height;
    constrain
}

/// A view that constrains its content to a given space.
#[derive(Rebuild)]
pub struct Constrain<V> {
    /// The content to constrain.
    pub content: Content<V>,
    /// The space to constrain the content to.
    #[rebuild(layout)]
    pub space: Space,
}

impl<V> Constrain<V> {
    /// Create a new constrained view.
    pub fn new(space: Space, content: V) -> Self {
        Self {
            content: Content::new(content),
            space,
        }
    }

    /// Create a new constrained view, with no bounds.
    pub fn unbounded(content: V) -> Self {
        Self {
            content: Content::new(content),
            space: Space::UNBOUNDED,
        }
    }
}

impl<T, V: View<T>> View<T> for Constrain<V> {
    type State = State<T, V>;

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
        let space = self.space.constrain(space);
        self.content.layout(state, cx, data, space)
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        self.content.draw(state, cx, data, canvas);
    }
}
