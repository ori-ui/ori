use gtk4::prelude::ButtonExt as _;

use crate::{Context, View};

pub fn button<V, T, A>(contents: V, on_click: impl FnMut(&mut T) -> A + 'static) -> Button<V, T>
where
    A: ori::IntoAction,
{
    Button::new(contents).on_click(on_click)
}

enum ButtonEvent {
    Clicked,
}

#[must_use]
pub struct Button<V, T> {
    contents: V,
    on_click: Box<dyn FnMut(&mut T) -> ori::Action>,
}

impl<V, T> Button<V, T> {
    pub fn new(contents: V) -> Self {
        Self {
            contents,
            on_click: Box::new(|_| ori::Action::new()),
        }
    }

    pub fn on_click<A>(mut self, mut on_click: impl FnMut(&mut T) -> A + 'static) -> Self
    where
        A: ori::IntoAction,
    {
        self.on_click = Box::new(move |data| on_click(data).into_action());
        self
    }
}

impl<V, T> ori::ViewMarker for Button<V, T> {}
impl<T, V: View<T>> ori::View<Context, T> for Button<V, T> {
    type Element = gtk4::Button;
    type State = (ori::ViewId, V::Element, V::State);

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (child, state) = self.contents.build(cx, data);

        let id = ori::ViewId::next();

        let button = gtk4::Button::new();
        button.set_child(Some(&child));

        button.connect_clicked({
            let cx = cx.clone();

            move |_| cx.event(ButtonEvent::Clicked, id)
        });

        (button, (id, child, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (_key, child, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        old: &mut Self,
    ) {
        self.contents.rebuild(
            child,
            state,
            cx,
            data,
            &mut old.contents,
        );

        if !super::is_parent(element, child) {
            element.set_child(Some(child));
        }
    }

    fn teardown(
        &mut self,
        _element: Self::Element,
        (_key, child, state): Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.contents.teardown(child, state, cx, data);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (key, child, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        let action = self.contents.event(child, state, cx, data, event);

        if !super::is_parent(element, child) {
            element.set_child(Some(child));
        }

        match event.take_targeted(*key) {
            Some(ButtonEvent::Clicked) => action | (self.on_click)(data),
            None => action,
        }
    }
}
