use crate::{Canvas, DrawCx, Event, EventCx, LayoutCx, RebuildCx, Size, Space};

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

    fn build(&self) -> Self::State;

    fn rebuild(&mut self, cx: &mut RebuildCx, old: &Self, state: &mut Self::State);

    fn event(&mut self, cx: &mut EventCx, state: &mut Self::State, data: &mut T, event: &Event);

    fn layout(&mut self, cx: &mut LayoutCx, state: &mut Self::State, space: Space) -> Size;

    fn draw(&mut self, cx: &mut DrawCx, state: &mut Self::State, canvas: &mut Canvas);
}
