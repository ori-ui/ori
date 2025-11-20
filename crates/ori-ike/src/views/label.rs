use ike::BuildCx;

use crate::Context;

pub fn label(text: impl ToString) -> Label {
    Label {
        text: text.to_string(),
    }
}

pub struct Label {
    text: String,
}

impl ori::ViewMarker for Label {}
impl<T> ori::View<Context, T> for Label {
    type Element = ike::WidgetId<ike::widgets::Label>;
    type State = ();

    fn build(&mut self, cx: &mut Context, _data: &mut T) -> (Self::Element, Self::State) {
        let element = ike::widgets::Label::new(cx, &self.text);

        (element, ())
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        _state: &mut Self::State,
        cx: &mut Context,
        _data: &mut T,
        old: &mut Self,
    ) {
        if self.text != old.text {
            ike::widgets::Label::set_text(cx, *element, &self.text);
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        _state: Self::State,
        cx: &mut Context,
        _data: &mut T,
    ) {
        cx.remove(element);
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut Context,
        _data: &mut T,
        _event: &mut ori::Event,
    ) -> ori::Action {
        ori::Action::new()
    }
}
