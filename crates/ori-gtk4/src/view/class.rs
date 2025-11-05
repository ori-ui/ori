use gtk4::prelude::WidgetExt as _;

use crate::{Context, View};

pub fn class<V>(class: impl ToString, content: V) -> Class<V> {
    Class::new(class, content)
}

#[must_use]
pub struct Class<V> {
    pub content: V,
    pub class: String,
}

impl<V> Class<V> {
    pub fn new(class: impl ToString, content: V) -> Self {
        Self {
            content,
            class: class.to_string(),
        }
    }
}

impl<T, V: View<T>> ori::View<Context, T> for Class<V> {
    type Element = V::Element;
    type State = V::State;

    fn build(
        &mut self,
        cx: &mut Context,
        data: &mut T,
    ) -> (Self::Element, Self::State) {
        let (element, state) = self.content.build(cx, data);

        element.set_css_classes(&[&self.class]);

        (element, state)
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        old: &mut Self,
    ) {
        self.content.rebuild(
            element,
            state,
            cx,
            data,
            &mut old.content,
        );

        if self.class != old.class {
            element.remove_css_class(&old.class);
            element.set_css_classes(&[&self.class]);
        }
    }

    fn teardown(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.content.teardown(element, state, cx, data);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        self.content.event(element, state, cx, data, event)
    }
}
