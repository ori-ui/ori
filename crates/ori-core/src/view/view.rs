use crate::{BuildCx, Canvas, DrawCx, Event, EventCx, LayoutCx, RebuildCx, Size, Space};

bitflags::bitflags! {
    #[must_use]
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct Update: u8 {
        const LAYOUT = 1 << 0;
        const DRAW = 1 << 1;
        const TREE = 1 << 2;
    }
}

pub trait View<T> {
    type State;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State;

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self);

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event);

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size;

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T, canvas: &mut Canvas);
}

impl<T> View<T> for () {
    type State = ();

    fn build(&mut self, _cx: &mut BuildCx, _data: &mut T) -> Self::State {}

    fn rebuild(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut RebuildCx,
        _data: &mut T,
        _old: &Self,
    ) {
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
        space.min
    }

    fn draw(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut DrawCx,
        _data: &mut T,
        _canvas: &mut Canvas,
    ) {
    }
}
