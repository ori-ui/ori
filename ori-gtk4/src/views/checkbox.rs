use gtk4::prelude::CheckButtonExt;

use crate::Context;

pub fn checkbox<T, A>(on_change: impl FnMut(&mut T, bool) -> A + 'static) -> Checkbox<T>
where
    A: Into<ori::Action>,
{
    Checkbox::new(on_change)
}

enum CheckboxEvent {
    Toggled(bool),
}

pub struct Checkbox<T> {
    checked:   Option<bool>,
    label:     Option<String>,
    on_change: Box<dyn FnMut(&mut T, bool) -> ori::Action>,
}

impl<T> Checkbox<T> {
    /// Create a new [`Checkbox`].
    pub fn new<A>(mut on_change: impl FnMut(&mut T, bool) -> A + 'static) -> Self
    where
        A: Into<ori::Action>,
    {
        Self {
            checked:   None,
            label:     None,
            on_change: Box::new(move |data, checked| on_change(data, checked).into()),
        }
    }

    /// Set the whether the [`Checkbox`] is `checked`.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    /// Set the label the [`Checkbox`].
    pub fn label(mut self, label: impl ToString) -> Self {
        self.label = Some(label.to_string());
        self
    }
}

impl<T> ori::ViewMarker for Checkbox<T> {}
impl<T> ori::View<Context, T> for Checkbox<T> {
    type Element = gtk4::CheckButton;
    type State = ori::ViewId;

    fn build(&mut self, cx: &mut Context, _data: &mut T) -> (Self::Element, Self::State) {
        let element = gtk4::CheckButton::new();

        if let Some(checked) = self.checked {
            element.set_active(checked);
        }

        let id = ori::ViewId::next();

        element.connect_toggled({
            let cx = cx.clone();

            move |element| {
                let checked = element.is_active();
                cx.event(CheckboxEvent::Toggled(checked), id)
            }
        });

        (element, id)
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut Context,
        _data: &mut T,
        old: &mut Self,
    ) {
        if let Some(checked) = self.checked
            && self.checked != old.checked
        {
            element.set_active(checked);
        }
    }

    fn teardown(&mut self, _element: Self::Element, _state: Self::State, _cx: &mut Context) {}

    fn event(
        &mut self,
        _element: &mut Self::Element,
        id: &mut Self::State,
        _cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        match event.take_targeted(*id) {
            Some(CheckboxEvent::Toggled(checked)) => (self.on_change)(data, checked),
            None => ori::Action::new(),
        }
    }
}
