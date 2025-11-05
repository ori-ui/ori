use gtk4::prelude::WidgetExt as _;

use crate::{Context, View};

pub fn expand<V>(expand: bool, content: V) -> Expand<V> {
    Expand::new(content).hexpand(expand).vexpand(expand)
}

pub fn hexpand<V>(expand: bool, content: V) -> Expand<V> {
    Expand::new(content).hexpand(expand)
}

pub fn vexpand<V>(expand: bool, content: V) -> Expand<V> {
    Expand::new(content).vexpand(expand)
}

#[must_use]
pub struct Expand<V> {
    pub content: V,
    pub hexpand: Option<bool>,
    pub vexpand: Option<bool>,
}

impl<V> Expand<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            hexpand: None,
            vexpand: None,
        }
    }

    pub fn hexpand(mut self, expand: bool) -> Self {
        self.hexpand = Some(expand);
        self
    }

    pub fn vexpand(mut self, expand: bool) -> Self {
        self.vexpand = Some(expand);
        self
    }
}

impl<T, V: View<T>> ori::View<Context, T> for Expand<V> {
    type Element = V::Element;
    type State = V::State;

    fn build(
        &mut self,
        cx: &mut Context,
        data: &mut T,
    ) -> (Self::Element, Self::State) {
        let (element, state) = self.content.build(cx, data);

        if let Some(hexpand) = self.hexpand {
            element.set_hexpand(hexpand);
        }

        if let Some(vexpand) = self.vexpand {
            element.set_vexpand(vexpand);
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

        if let Some(hexpand) = self.hexpand {
            element.set_hexpand(hexpand);
        }

        if let Some(vexpand) = self.vexpand {
            element.set_vexpand(vexpand);
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
