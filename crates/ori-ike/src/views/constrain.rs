use ike::{BuildCx, Size, Transition};

use crate::{Context, View};

pub fn constrain<V>(contents: V) -> Constrain<V> {
    Constrain::new(contents)
}

pub fn min_size<V>(min_size: impl Into<Size>, contents: V) -> Constrain<V> {
    Constrain::new(contents).min_size(min_size.into())
}

pub fn max_size<V>(max_size: impl Into<Size>, contents: V) -> Constrain<V> {
    Constrain::new(contents).max_size(max_size.into())
}

pub fn size<V>(size: impl Into<Size>, contents: V) -> Constrain<V> {
    Constrain::new(contents).size(size.into())
}

pub fn min_width<V>(min_width: f32, contents: V) -> Constrain<V> {
    Constrain::new(contents).min_width(min_width)
}

pub fn min_height<V>(min_height: f32, contents: V) -> Constrain<V> {
    Constrain::new(contents).min_height(min_height)
}

pub fn max_width<V>(max_width: f32, contents: V) -> Constrain<V> {
    Constrain::new(contents).max_width(max_width)
}

pub fn max_height<V>(max_height: f32, contents: V) -> Constrain<V> {
    Constrain::new(contents).max_height(max_height)
}

pub fn width<V>(width: f32, contents: V) -> Constrain<V> {
    Constrain::new(contents).width(width)
}

pub fn height<V>(height: f32, contents: V) -> Constrain<V> {
    Constrain::new(contents).height(height)
}

pub struct Constrain<V> {
    contents:            V,
    min_size:            Size,
    max_size:            Size,
    min_size_transition: Transition,
    max_size_transition: Transition,
}

impl<V> Constrain<V> {
    pub fn new(contents: V) -> Self {
        Self {
            contents,
            min_size: Size::all(0.0),
            max_size: Size::all(f32::INFINITY),
            min_size_transition: Transition::INSTANT,
            max_size_transition: Transition::INSTANT,
        }
    }

    pub fn min_size_transition(mut self, transition: Transition) -> Self {
        self.min_size_transition = transition;
        self
    }

    pub fn max_size_transition(mut self, transition: Transition) -> Self {
        self.max_size_transition = transition;
        self
    }

    pub fn transition(mut self, transition: Transition) -> Self {
        self.min_size_transition = transition;
        self.max_size_transition = transition;
        self
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
        let (contents, state) = self.contents.build(cx, data);

        let mut widget = ike::widgets::Constrain::new(cx, contents);

        ike::widgets::Constrain::set_min_size(&mut widget, self.min_size);
        ike::widgets::Constrain::set_max_size(&mut widget, self.max_size);
        ike::widgets::Constrain::set_min_size_transition(&mut widget, self.min_size_transition);
        ike::widgets::Constrain::set_max_size_transition(&mut widget, self.max_size_transition);

        (widget.id(), (contents, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        old: &mut Self,
    ) {
        self.contents.rebuild(
            contents,
            state,
            cx,
            data,
            &mut old.contents,
        );

        let mut widget = cx.get_mut(*element);

        if !widget.is_child(*contents) {
            ike::widgets::Constrain::set_child(&mut widget, *contents);
        }

        if self.min_size != old.min_size {
            ike::widgets::Constrain::set_min_size(&mut widget, self.min_size);
        }

        if self.max_size != old.max_size {
            ike::widgets::Constrain::set_max_size(&mut widget, self.max_size);
        }

        if self.min_size_transition != old.min_size_transition {
            ike::widgets::Constrain::set_min_size_transition(&mut widget, self.min_size_transition);
        }

        if self.max_size_transition != old.max_size_transition {
            ike::widgets::Constrain::set_max_size_transition(&mut widget, self.max_size_transition);
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (contents, state): Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.contents.teardown(contents, state, cx, data);
        cx.remove(element);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        let action = self.contents.event(contents, state, cx, data, event);

        let mut widget = cx.get_mut(*element);

        if !widget.is_child(*contents) {
            ike::widgets::Constrain::set_child(&mut widget, *contents);
        }

        action
    }
}
