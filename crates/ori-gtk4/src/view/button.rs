use gtk4::prelude::{ButtonExt as _, WidgetExt};

use crate::{Context, View};

pub fn button<V, T, A>(
    content: V,
    on_click: impl FnMut(&mut T) -> A + 'static,
) -> Button<V, T>
where
    A: ori::IntoAction,
{
    Button::new(content).on_click(on_click)
}

enum ButtonEvent {
    Clicked,
}

#[must_use]
pub struct Button<V, T> {
    pub content: V,
    pub on_click: Box<dyn FnMut(&mut T) -> ori::Action>,
}

impl<V, T> Button<V, T> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            on_click: Box::new(|_| ori::Action::none()),
        }
    }

    pub fn on_click<A>(
        mut self,
        mut on_click: impl FnMut(&mut T) -> A + 'static,
    ) -> Self
    where
        A: ori::IntoAction,
    {
        self.on_click = Box::new(move |data| on_click(data).into_action());
        self
    }
}

impl<T, V: View<T>> ori::View<Context, T> for Button<V, T> {
    type Element = gtk4::Button;
    type State = (ori::ViewId, V::Element, V::State);

    fn build(
        &mut self,
        cx: &mut Context,
        data: &mut T,
    ) -> (Self::Element, Self::State) {
        let (child, state) = self.content.build(cx, data);

        let id = ori::ViewId::new();

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
        (_id, child, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        old: &mut Self,
    ) {
        self.content.rebuild(child, state, cx, data, &mut old.content);

        if !child.is_ancestor(element) {
            element.set_child(Some(child));
        }
    }

    fn teardown(
        &mut self,
        _element: &mut Self::Element,
        (_id, child, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.content.teardown(child, state, cx, data);
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        (id, child, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        let action = self.content.event(child, state, cx, data, event);

        match event.get_targeted(*id) {
            Some(ButtonEvent::Clicked) => action | (self.on_click)(data),

            _ => action,
        }
    }
}
