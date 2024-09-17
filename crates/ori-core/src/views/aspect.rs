use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    rebuild::Rebuild,
    view::{Pod, State, View},
};

/// Create a new [`Aspect`] view.
pub fn aspect<T>(aspect: f32, view: impl View<T>) -> Aspect<impl View<T>> {
    Aspect::new(aspect, view)
}

/// A view that lays out its content with a fixed aspect ratio.
#[derive(Rebuild)]
pub struct Aspect<V> {
    /// The content.
    pub content: Pod<V>,

    /// The aspect ratio of the content.
    #[rebuild(layout)]
    pub aspect: f32,
}

impl<V> Aspect<V> {
    /// Creates a new `Aspect` view.
    pub fn new(content: V, aspect: f32) -> Self {
        Self {
            content: Pod::new(content),
            aspect,
        }
    }
}

impl<T, V: View<T>> View<T> for Aspect<V> {
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
        let mut new_width = space.max.width;
        let mut new_height = space.max.height;

        if new_width.is_infinite() {
            new_width = new_height * self.aspect;
        } else if new_height.is_infinite() {
            new_height = new_width / self.aspect;
        }

        if new_width > space.max.width {
            new_width = space.max.width;
            new_height = new_width / self.aspect;
        }

        if new_height > space.max.height {
            new_height = space.max.height;
            new_width = new_height * self.aspect;
        }

        if new_width < space.min.width {
            new_width = space.min.width;
            new_height = new_width / self.aspect;
        }

        if new_height < space.min.height {
            new_height = space.min.height;
            new_width = new_height * self.aspect;
        }

        let space = Space::from_size(Size::new(new_width, new_height));
        self.content.layout(state, cx, data, space)
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        self.content.draw(state, cx, data);
    }
}
