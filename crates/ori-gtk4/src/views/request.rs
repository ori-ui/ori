use gtk4::prelude::WidgetExt;

use crate::{Context, View};

pub fn request_size<V>(width: u32, height: u32, content: V) -> Request<V> {
    Request::new(content).width(width).height(height)
}

pub fn request_width<V>(width: u32, content: V) -> Request<V> {
    Request::new(content).width(width)
}

pub fn request_height<V>(height: u32, content: V) -> Request<V> {
    Request::new(content).height(height)
}

pub struct Request<V> {
    pub content: V,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl<V> Request<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            width: None,
            height: None,
        }
    }

    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }
}

impl<T, V> ori::View<Context, T> for Request<V>
where
    V: View<T>,
{
    type Element = V::Element;
    type State = V::State;

    fn build(
        &mut self,
        cx: &mut Context,
        data: &mut T,
    ) -> (Self::Element, Self::State) {
        let (element, state) = self.content.build(cx, data);

        element.set_size_request(
            self.width.map_or(-1, |w| w as i32),
            self.height.map_or(-1, |h| h as i32),
        );

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

        if self.width != old.width || self.height != old.height {
            element.set_size_request(
                self.width.map_or(-1, |w| w as i32),
                self.height.map_or(-1, |h| h as i32),
            );
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
