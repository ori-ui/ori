use ike::{BuildCx, Size};

use crate::{Context, View};

pub fn constrain<V>(content: V) -> Constrain<V> {
    Constrain::new(content)
}

pub fn min_size<V>(min_size: Size, content: V) -> Constrain<V> {
    Constrain::new(content).min_size(min_size)
}

pub fn max_size<V>(max_size: Size, content: V) -> Constrain<V> {
    Constrain::new(content).max_size(max_size)
}

pub fn min_width<V>(min_width: f32, content: V) -> Constrain<V> {
    Constrain::new(content).min_width(min_width)
}

pub fn min_height<V>(min_height: f32, content: V) -> Constrain<V> {
    Constrain::new(content).min_height(min_height)
}

pub fn max_width<V>(max_width: f32, content: V) -> Constrain<V> {
    Constrain::new(content).max_width(max_width)
}

pub fn max_height<V>(max_height: f32, content: V) -> Constrain<V> {
    Constrain::new(content).max_height(max_height)
}

pub fn size<V>(size: Size, content: V) -> Constrain<V> {
    Constrain::new(content).size(size)
}

pub fn width<V>(width: f32, content: V) -> Constrain<V> {
    Constrain::new(content).width(width)
}

pub fn height<V>(height: f32, content: V) -> Constrain<V> {
    Constrain::new(content).height(height)
}

pub struct Constrain<V> {
    content:  V,
    min_size: Size,
    max_size: Size,
}

impl<V> Constrain<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            min_size: Size::all(0.0),
            max_size: Size::all(f32::INFINITY),
        }
    }

    pub fn min_size(mut self, min_size: Size) -> Self {
        self.min_size = min_size;
        self
    }

    pub fn max_size(mut self, max_size: Size) -> Self {
        self.max_size = max_size;
        self
    }

    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min_size.width = min_width;
        self
    }

    pub fn min_height(mut self, min_height: f32) -> Self {
        self.min_size.height = min_height;
        self
    }

    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_size.width = max_width;
        self
    }

    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_size.height = max_height;
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.min_size = size;
        self.max_size = size;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.min_size.width = width;
        self.max_size.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.min_size.height = height;
        self.max_size.height = height;
        self
    }
}

impl<V> ori::ViewMarker for Constrain<V> {}
impl<T, V> ori::View<Context, T> for Constrain<V>
where
    V: View<T>,
{
    type Element = ike::WidgetId<ike::widgets::Constrain>;
    type State = (V::Element, V::State);

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (content, state) = self.content.build(cx, data);

        let mut widget = ike::widgets::Constrain::new(cx, content);

        ike::widgets::Constrain::set_min_size(&mut widget, self.min_size);
        ike::widgets::Constrain::set_max_size(&mut widget, self.max_size);

        (widget.id(), (content, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (content, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        old: &mut Self,
    ) {
        self.content.rebuild(
            content,
            state,
            cx,
            data,
            &mut old.content,
        );

        let mut widget = cx.get_mut(*element);

        if !widget.is_child(*content) {
            ike::widgets::Constrain::set_child(&mut widget, *content);
        }

        if self.min_size != old.min_size {
            ike::widgets::Constrain::set_min_size(&mut widget, self.min_size);
        }

        if self.max_size != old.max_size {
            ike::widgets::Constrain::set_max_size(&mut widget, self.max_size);
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (content, state): Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.content.teardown(content, state, cx, data);
        cx.remove(element);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (content, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        let action = self.content.event(content, state, cx, data, event);

        let mut widget = cx.get_mut(*element);

        if !widget.is_child(*content) {
            ike::widgets::Constrain::set_child(&mut widget, *content);
        }

        action
    }
}
