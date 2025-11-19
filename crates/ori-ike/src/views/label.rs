use crate::Context;

pub struct Label {}

impl ori::ViewMarker for Label {}
impl<T> ori::View<Context, T> for Label {
    type Element = ike::WidgetId<Label>;
    type State = ();

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        cx.build();

        todo!()
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        old: &mut Self,
    ) {
        todo!()
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        state: Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        todo!()
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        todo!()
    }
}
