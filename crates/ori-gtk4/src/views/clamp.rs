use gtk4::prelude::OrientableExt;

use crate::{Context, View, views::Axis};

pub fn clamp<V>(axis: Axis, size: u32, contents: V) -> Clamp<V> {
    Clamp::new(axis, size, contents)
}

pub fn clamp_width<V>(size: u32, contents: V) -> Clamp<V> {
    Clamp::new(Axis::Horizontal, size, contents)
}

pub fn clamp_height<V>(size: u32, contents: V) -> Clamp<V> {
    Clamp::new(Axis::Vertical, size, contents)
}

pub struct Clamp<V> {
    contents: V,
    axis:     Axis,
    size:     u32,
}

impl<V> Clamp<V> {
    pub fn new(axis: Axis, size: u32, contents: V) -> Self {
        Self {
            contents,
            axis,
            size,
        }
    }
}

impl<V> ori::ViewMarker for Clamp<V> {}
impl<T, V> ori::View<Context, T> for Clamp<V>
where
    V: View<T>,
{
    type Element = libadwaita::Clamp;
    type State = (V::Element, V::State);

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (child, state) = self.contents.build(cx, data);

        let element = libadwaita::Clamp::new();
        element.set_child(Some(&child));
        element.set_orientation(self.axis.into());
        element.set_maximum_size(self.size as i32);

        (element, (child, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (child, state): &mut Self::State,
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

        if self.axis != old.axis {
            element.set_orientation(self.axis.into());
        }

        if self.size != old.size {
            element.set_maximum_size(self.size as i32);
        }
    }

    fn teardown(
        &mut self,
        _element: Self::Element,
        (child, state): Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.contents.teardown(child, state, cx, data);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (child, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        let action = self.contents.event(child, state, cx, data, event);

        if !super::is_parent(element, child) {
            element.set_child(Some(child));
        }

        action
    }
}
