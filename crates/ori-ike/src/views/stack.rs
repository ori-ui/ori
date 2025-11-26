use ike::{
    AnyWidgetId, BuildCx, WidgetMut,
    widgets::{Align, Axis, Justify},
};
use ori::ElementSeq;

use crate::{Context, View};

pub fn stack<V>(axis: Axis, contents: V) -> Stack<V> {
    Stack::new(axis, contents)
}

pub fn hstack<V>(contents: V) -> Stack<V> {
    stack(Axis::Horizontal, contents)
}

pub fn vstack<V>(contents: V) -> Stack<V> {
    stack(Axis::Vertical, contents)
}

pub struct Stack<V> {
    contents: V,
    axis:     Axis,
    justify:  Justify,
    align:    Align,
    gap:      f32,
}

impl<V> Stack<V> {
    pub fn new(axis: Axis, contents: V) -> Self {
        Self {
            contents,
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
        let (children, states) = self.contents.seq_build(cx, data);

        let mut widget = ike::widgets::Stack::new(cx);

        ike::widgets::Stack::set_axis(&mut widget, self.axis);
        ike::widgets::Stack::set_justify(&mut widget, self.justify);
        ike::widgets::Stack::set_align(&mut widget, self.align);
        ike::widgets::Stack::set_gap(&mut widget, self.gap);

        for (i, child) in children.iter().enumerate() {
            widget.add_child(child.contents);
            ike::widgets::Stack::set_flex(&mut widget, i, child.flex, child.tight);
        }

        (widget.id(), (children, states))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (children, states): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        old: &mut Self,
    ) {
        self.contents.seq_rebuild(
            children,
            states,
            cx,
            data,
            &mut old.contents,
        );

        let mut widget = cx.get_mut(*element);
        update_children(&mut widget, children);

        if self.axis != old.axis {
            ike::widgets::Stack::set_axis(&mut widget, self.axis);
        }

        if self.justify != old.justify {
            ike::widgets::Stack::set_justify(&mut widget, self.justify);
        }

        if self.align != old.align {
            ike::widgets::Stack::set_align(&mut widget, self.align);
        }

        if self.gap != old.gap {
            ike::widgets::Stack::set_gap(&mut widget, self.gap);
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (children, states): Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.contents.seq_teardown(children, states, cx, data);
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
        let action = self.contents.seq_event(children, states, cx, data, event);

        let mut widget = cx.get_mut(*element);
        update_children(&mut widget, children);

        action
    }
}

fn update_children(
    widget: &mut WidgetMut<ike::widgets::Stack>,
    children: &mut impl ElementSeq<Flex<ike::WidgetId>>,
) {
    for child in children.iter() {
        if !widget.is_child(child.contents) {
            widget.add_child(child.contents);
        }
    }

    for (i, child) in children.iter().enumerate() {
        if widget.children()[i] != child.contents {
            widget.swap_children(i, i + 1);
        }

        let (flex, tight) = ike::widgets::Stack::get_flex(widget, i);

        if child.flex != flex || child.tight != tight {
            ike::widgets::Stack::set_flex(widget, i, child.flex, child.tight);
        }
    }

    let count = widget.children().len();

    for i in (children.count()..count).rev() {
        widget.remove_child(i);
    }
}

pub fn flex<V>(flex: f32, contents: V) -> Flex<V> {
    Flex::new(flex, true, contents)
}

pub fn expand<V>(flex: f32, contents: V) -> Flex<V> {
    Flex::new(flex, false, contents)
}

#[derive(Clone, Copy)]
pub struct Flex<V> {
    contents: V,
    flex:     f32,
    tight:    bool,
}

impl<V> Flex<V> {
    pub fn new(flex: f32, tight: bool, contents: V) -> Self {
        Self {
            contents,
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
        let (element, state) = self.contents.build(cx, data);
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
        self.contents.rebuild(
            &mut element.contents,
            state,
            cx,
            data,
            &mut old.contents,
        );
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        state: Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.contents.teardown(element.contents, state, cx, data);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        self.contents.event(
            &mut element.contents,
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
            contents: sub.upcast(),
            flex:     0.0,
            tight:    false,
        }
    }

    fn downcast(self) -> S {
        self.contents.downcast()
    }

    fn downcast_with<T>(&mut self, f: impl FnOnce(&mut S) -> T) -> T {
        self.contents.downcast_with(f)
    }
}

impl<S> ori::Super<Context, Flex<S>> for Flex<ike::WidgetId>
where
    S: AnyWidgetId,
{
    fn upcast(_cx: &mut Context, sub: Flex<S>) -> Self {
        Self {
            contents: sub.contents.upcast(),
            flex:     sub.flex,
            tight:    sub.tight,
        }
    }

    fn downcast(self) -> Flex<S> {
        Flex {
            contents: self.contents.downcast(),
            flex:     self.flex,
            tight:    self.tight,
        }
    }

    fn downcast_with<T>(&mut self, f: impl FnOnce(&mut Flex<S>) -> T) -> T {
        let mut flex: Flex<S> = self.downcast();
        let output = f(&mut flex);
        self.contents = flex.contents.upcast();
        self.flex = flex.flex;
        output
    }
}
