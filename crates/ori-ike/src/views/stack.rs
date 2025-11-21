use ike::{
    AnyWidgetId, BuildCx,
    widgets::{Align, Axis, Justify},
};
use ori::ElementSeq;

use crate::{Context, View};

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
    V: ori::ViewSeq<Context, T, Flex<ike::WidgetId>>,
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

        for (i, child) in children.iter().enumerate() {
            cx.add_child(element, child.content);
            ike::widgets::Stack::set_flex(cx, element, i, child.flex, child.tight);
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
    children: &mut impl ElementSeq<Flex<ike::WidgetId>>,
) {
    for child in children.iter() {
        if !cx.is_parent(element, child.content) {
            cx.add_child(element, child.content);
        }
    }

    for (i, child) in children.iter().enumerate() {
        if cx.children(element)[i] != child.content {
            cx.swap_children(element, i, i + 1);
        }

        let (flex, tight) = ike::widgets::Stack::get_flex(cx, element, i);

        if child.flex != flex || child.tight != tight {
            ike::widgets::Stack::set_flex(cx, element, i, child.flex, child.tight);
        }
    }

    let count = cx.children(element).len();

    for i in (children.count()..count).rev() {
        cx.remove_child(element, i);
    }
}

pub fn flex<V>(flex: f32, content: V) -> Flex<V> {
    Flex::new(flex, true, content)
}

pub fn expand<V>(flex: f32, content: V) -> Flex<V> {
    Flex::new(flex, false, content)
}

#[derive(Clone, Copy)]
pub struct Flex<V> {
    content: V,
    flex:    f32,
    tight:   bool,
}

impl<V> Flex<V> {
    pub fn new(flex: f32, tight: bool, content: V) -> Self {
        Self {
            content,
            flex,
            tight,
        }
    }
}

impl<V> ori::ViewMarker for Flex<V> {}
impl<T, V> ori::View<Context, T> for Flex<V>
where
    V: View<T>,
{
    type Element = Flex<V::Element>;
    type State = V::State;

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (element, state) = self.content.build(cx, data);
        let element = Flex::new(self.flex, self.tight, element);

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
            &mut element.content,
            state,
            cx,
            data,
            &mut old.content,
        );
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        state: Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.content.teardown(element.content, state, cx, data);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        self.content.event(
            &mut element.content,
            state,
            cx,
            data,
            event,
        )
    }
}

impl<S> ori::Super<Context, S> for Flex<ike::WidgetId>
where
    S: AnyWidgetId,
{
    fn upcast(_cx: &mut Context, sub: S) -> Self {
        Flex {
            content: sub.upcast(),
            flex:    0.0,
            tight:   false,
        }
    }

    fn downcast(self) -> S {
        self.content.downcast()
    }

    fn downcast_with<T>(&mut self, f: impl FnOnce(&mut S) -> T) -> T {
        self.content.downcast_with(f)
    }
}

impl<S> ori::Super<Context, Flex<S>> for Flex<ike::WidgetId>
where
    S: AnyWidgetId,
{
    fn upcast(_cx: &mut Context, sub: Flex<S>) -> Self {
        Self {
            content: sub.content.upcast(),
            flex:    sub.flex,
            tight:   sub.tight,
        }
    }

    fn downcast(self) -> Flex<S> {
        Flex {
            content: self.content.downcast(),
            flex:    self.flex,
            tight:   self.tight,
        }
    }

    fn downcast_with<T>(&mut self, f: impl FnOnce(&mut Flex<S>) -> T) -> T {
        let mut flex: Flex<S> = self.downcast();
        let output = f(&mut flex);
        self.content = flex.content.upcast();
        self.flex = flex.flex;
        output
    }
}
