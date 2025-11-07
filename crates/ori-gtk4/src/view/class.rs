use gtk4::prelude::WidgetExt as _;

use crate::{Context, View};

pub fn class<V>(classes: &[impl ToString], content: V) -> Class<V> {
    Class::new(classes, content)
}

#[must_use]
pub struct Class<V> {
    pub content: V,
    pub classes: Vec<String>,
}

impl<V> Class<V> {
    pub fn new(classes: &[impl ToString], content: V) -> Self {
        Self {
            content,
            classes: classes.iter().map(|s| s.to_string()).collect(),
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

        for class in &self.classes {
            element.add_css_class(class);
        }

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

        for class in &old.classes {
            if !self.classes.contains(class) {
                element.remove_css_class(class);
            }
        }

        for class in &self.classes {
            if !old.classes.contains(class) {
                element.add_css_class(class);
            }
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        state: Self::State,
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
