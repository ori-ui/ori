use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    image::Image,
    layout::{Point, Size, Space},
    view::View,
};

impl<T> View<T> for Image {
    type State = ();

    fn build(&mut self, _cx: &mut BuildCx, _data: &mut T) -> Self::State {}

    fn rebuild(&mut self, _state: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        if self != old {
            cx.request_layout();
            cx.request_draw();
        }
    }

    fn event(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut EventCx,
        _data: &mut T,
        _event: &Event,
    ) {
    }

    fn layout(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        space.fit(self.size())
    }

    fn draw(&mut self, _state: &mut Self::State, cx: &mut DrawCx, _data: &mut T) {
        cx.image(Point::ZERO, self.clone());
    }
}
