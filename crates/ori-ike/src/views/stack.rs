use ike::{
    BuildCx,
    widgets::{Align, Axis, Justify},
};
use ori::ElementSeq;

use crate::Context;

pub fn stack<V>(axis: Axis, content: V) -> Stack<V> {
    Stack::new(axis, content)
}

pub fn hstack<V>(content: V) -> Stack<V> {
    stack(Axis::Horizontal, content)
}

pub fn vstack<V>(content: V) -> Stack<V> {
    stack(Axis::Vertical, content)
}

pub struct Stack<V> {
    content: V,
    axis:    Axis,
    justify: Justify,
    align:   Align,
    gap:     f32,
}

impl<V> Stack<V> {
    pub fn new(axis: Axis, content: V) -> Self {
        Self {
            content,
            axis,
            justify: Justify::Start,
            align: Align::Center,
            gap: 0.0,
        }
    }

    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }
}

impl<V> ori::ViewMarker for Stack<V> {}
impl<T, V> ori::View<Context, T> for Stack<V>
where
    V: ori::ViewSeq<Context, T, ike::WidgetId>,
{
    type Element = ike::WidgetId<ike::widgets::Stack>;
    type State = (V::Elements, V::States);

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (children, states) = self.content.seq_build(cx, data);

        let element = ike::widgets::Stack::new(cx);

        ike::widgets::Stack::set_axis(cx, element, self.axis);
        ike::widgets::Stack::set_justify(cx, element, self.justify);
        ike::widgets::Stack::set_align(cx, element, self.align);
        ike::widgets::Stack::set_gap(cx, element, self.gap);

        for child in children.iter() {
            ike::widgets::Stack::add_flex_child(cx, element, child, 0.0);
        }

        (element, (children, states))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (children, states): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        old: &mut Self,
    ) {
        self.content.seq_rebuild(
            children,
            states,
            cx,
            data,
            &mut old.content,
        );

        update_children(cx, *element, children);

        if self.axis != old.axis {
            ike::widgets::Stack::set_axis(cx, *element, self.axis);
        }

        if self.justify != old.justify {
            ike::widgets::Stack::set_justify(cx, *element, self.justify);
        }

        if self.align != old.align {
            ike::widgets::Stack::set_align(cx, *element, self.align);
        }

        if self.gap != old.gap {
            ike::widgets::Stack::set_gap(cx, *element, self.gap);
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (children, states): Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.content.seq_teardown(children, states, cx, data);
        cx.remove(element);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (children, states): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        let action = self.content.seq_event(children, states, cx, data, event);

        update_children(cx, *element, children);

        action
    }
}

fn update_children(
    cx: &mut impl BuildCx,
    element: ike::WidgetId<ike::widgets::Stack>,
    children: &mut impl ElementSeq<ike::WidgetId>,
) {
    for child in children.iter() {
        if !cx.is_parent(element, child) {
            ike::widgets::Stack::add_flex_child(cx, element, child, 0.0);
        }
    }

    for (i, &child) in children.iter().enumerate() {
        if cx.children(element)[i] != child {
            cx.swap_children(element, i, i + 1);
        }
    }

    let count = cx.children(element).len();

    for i in (children.count()..count).rev() {
        cx.remove_child(element, i);
    }
}
