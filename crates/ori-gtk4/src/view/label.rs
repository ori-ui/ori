use crate::Context;

pub fn label(text: impl ToString) -> Label {
    Label::new(text)
}

#[must_use]
pub struct Label {
    pub text: String,
}

impl Label {
    pub fn new(text: impl ToString) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

impl<T> ori::View<Context, T> for Label {
    type Element = gtk4::Label;
    type State = ();

    fn build(
        &mut self,
        _cx: &mut Context,
        _data: &mut T,
    ) -> (Self::Element, Self::State) {
        let text = gtk4::Label::new(Some(&self.text));

        (text, ())
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut Context,
        _data: &mut T,
        old: &mut Self,
    ) {
        if self.text != old.text {
            element.set_text(&self.text);
        }
    }

    fn teardown(
        &mut self,
        _element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut Context,
        _data: &mut T,
    ) {
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut Context,
        _data: &mut T,
        _event: &mut ori::Event,
    ) -> ori::Action {
        ori::Action::none()
    }
}
