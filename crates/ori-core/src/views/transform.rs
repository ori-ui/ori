use ori_macro::example;

use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Affine, Size, Space, Vector},
    rebuild::Rebuild,
    view::{Pod, State, View},
};

/// Create a new [`Transform`] view.
pub fn transform<V>(transform: Affine, view: V) -> Transform<V> {
    Transform::new(transform, view)
}

/// Create a new [`Transform`] view that translates its content.
pub fn translate<V>(translation: impl Into<Vector>, view: V) -> Transform<V> {
    Transform::new(Affine::translate(translation.into()), view)
}

/// Create a new [`Transform`] view that rotates its content.
pub fn rotate<V>(degrees: f32, view: V) -> Transform<V> {
    Transform::new(Affine::rotate(degrees), view)
}

/// Create a new [`Transform`] view that rotates its content in degrees.
pub fn rotate_degrees<V>(degrees: f32, view: V) -> Transform<V> {
    rotate(degrees.to_radians(), view)
}

/// Create a new [`Transform`] view that scales its content.
pub fn scale<V>(scale: impl Into<Vector>, view: V) -> Transform<V> {
    Transform::new(Affine::scale(scale.into()), view)
}

/// A view that transforms its content.
#[example(name = "transform", width = 400, height = 300)]
#[derive(Rebuild)]
pub struct Transform<V> {
    /// The content.
    pub content: Pod<V>,
    /// The transform.
    #[rebuild(layout)]
    pub transform: Affine,
}

impl<V> Transform<V> {
    /// Create a new [`Transform`] view.
    pub fn new(transform: Affine, content: V) -> Self {
        Self {
            content: Pod::new(content),
            transform,
        }
    }
}

impl<T, V: View<T>> View<T> for Transform<V> {
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
        let size = self.content.layout(state, cx, data, space);
        let center = Affine::translate(size.to_vector() / 2.0);
        state.set_transform(center * self.transform * center.inverse());

        size
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        self.content.draw(state, cx, data);
    }
}
