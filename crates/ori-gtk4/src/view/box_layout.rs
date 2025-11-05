use gtk4::prelude::{BoxExt as _, OrientableExt as _, WidgetExt};
use ori::Event;

use crate::Context;

pub fn box_layout<V>(axis: Axis, content: V) -> BoxLayout<V> {
    BoxLayout::new(axis, content)
}

pub fn row<V>(content: V) -> BoxLayout<V> {
    BoxLayout::new(Axis::Horizontal, content)
}

pub fn column<V>(content: V) -> BoxLayout<V> {
    BoxLayout::new(Axis::Vertical, content)
}

#[macro_export]
macro_rules! row {
    [$($view:expr),* $(,)?] => {
        $crate::row(($($view,)*))
    };
}

#[macro_export]
macro_rules! column {
    [$($view:expr),* $(,)?] => {
        $crate::column(($($view,)*))
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl From<Axis> for gtk4::Orientation {
    fn from(axis: Axis) -> Self {
        match axis {
            Axis::Horizontal => gtk4::Orientation::Horizontal,
            Axis::Vertical => gtk4::Orientation::Vertical,
        }
    }
}

#[must_use]
pub struct BoxLayout<V> {
    pub content: V,
    pub spacing: u32,
    pub axis: Axis,
}

impl<V> BoxLayout<V> {
    pub fn new(axis: Axis, content: V) -> Self {
        Self {
            content,
            spacing: 0,
            axis,
        }
    }

    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl<T, V> ori::View<Context, T> for BoxLayout<V>
where
    V: ori::ViewSeq<Context, gtk4::Widget, T>,
{
    type Element = gtk4::Box;
    type State = (Vec<gtk4::Widget>, V::SeqState);

    fn build(
        &mut self,
        cx: &mut Context,
        data: &mut T,
    ) -> (Self::Element, Self::State) {
        let (children, state) = self.content.seq_build(cx, data);

        let element = gtk4::Box::new(self.axis.into(), self.spacing as i32);

        for child in &children {
            element.append(child);
        }

        (element, (children, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (children, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        old: &mut Self,
    ) {
        self.content.seq_rebuild(
            children,
            state,
            cx,
            data,
            &mut old.content,
        );

        for child in children.iter() {
            if !child.is_ancestor(element) {
                element.append(child);
            }
        }

        if self.spacing != old.spacing {
            element.set_spacing(self.spacing as i32);
        }

        if self.axis != old.axis {
            element.set_orientation(self.axis.into());
        }
    }

    fn teardown(
        &mut self,
        _element: &mut Self::Element,
        (children, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.content.seq_teardown(children, state, cx, data);
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        (children, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut Event,
    ) -> ori::Action {
        self.content.seq_event(children, state, cx, data, event)
    }
}
