use gtk4::prelude::FrameExt as _;

use crate::{Context, View};

pub fn frame<V>(content: V) -> Frame<V> {
    Frame::new(content)
}

#[must_use]
pub struct Frame<V> {
    pub content: V,
    pub label: Option<String>,
}

impl<V> Frame<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            label: None,
        }
    }

    pub fn label(mut self, label: impl ToString) -> Self {
        self.label = Some(label.to_string());
        self
    }
}

impl<T, V: View<T>> ori::View<Context, T> for Frame<V> {
    type Element = gtk4::Frame;
    type State = (V::Element, V::State);

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (child, state) = self.content.build(cx, data);

        let element = gtk4::Frame::new(self.label.as_deref());
        element.set_child(Some(&child));

        (element, (child, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (child, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        old: &mut Self,
    ) -> bool {
        let changed = self
            .content
            .rebuild(child, state, cx, data, &mut old.content);

        if changed && !super::is_parent(element, child) {
            element.set_child(Some(child));
        }

        if self.label != old.label {
            element.set_label(self.label.as_deref());
        }

        false
    }

    fn teardown(
        &mut self,
        _element: Self::Element,
        (child, state): Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.content.teardown(child, state, cx, data);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (child, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> (bool, ori::Action) {
        let (changed, action) = self.content.event(child, state, cx, data, event);

        if changed && !super::is_parent(element, child) {
            element.set_child(Some(child));
        }

        (false, action)
    }
}
