use std::ops::{Deref, DerefMut};

use gtk4::{
    glib::object::Cast as _,
    prelude::{BoxExt as _, OrientableExt as _, WidgetExt as _},
};
use ori::{ElementSeq, Event};

use crate::{Context, ViewSeq, views::Axis};

pub fn line<V>(axis: Axis, content: V) -> Line<V> {
    Line::new(axis, content)
}

pub fn hline<V>(content: V) -> Line<V> {
    Line::new(Axis::Horizontal, content)
}

pub fn vline<V>(content: V) -> Line<V> {
    Line::new(Axis::Vertical, content)
}

#[must_use]
pub struct Line<V> {
    pub content: V,
    pub spacing: u32,
    pub axis: Axis,
}

impl<V> Line<V> {
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

impl<V> Deref for Line<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<V> DerefMut for Line<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

impl<V> ori::ViewMarker for Line<V> {}
impl<T, V> ori::View<Context, T> for Line<V>
where
    V: ViewSeq<T>,
{
    type Element = gtk4::Box;
    type State = (V::Elements, V::States);

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (mut children, state) = self.content.seq_build(cx, data);

        let element = gtk4::Box::default();
        element.set_orientation(self.axis.into());
        element.set_spacing(self.spacing as i32);

        for child in children.element_iter() {
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
    ) -> bool {
        let changed = self.content.seq_rebuild(
            children,
            state,
            cx,
            data,
            &mut old.content,
        );

        update_children(element, children, &changed);

        // update state
        if self.spacing != old.spacing {
            element.set_spacing(self.spacing as i32);
        }

        if self.axis != old.axis {
            element.set_orientation(self.axis.into());
        }

        false
    }

    fn teardown(
        &mut self,
        _element: Self::Element,
        (children, state): Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.content.seq_teardown(children, state, cx, data);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (children, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut Event,
    ) -> (bool, ori::Action) {
        let (changed, action) = self.content.seq_event(children, state, cx, data, event);

        update_children(element, children, &changed);

        (false, action)
    }
}

fn update_children(
    element: &gtk4::Box,
    children: &mut impl ElementSeq<gtk4::Widget>,
    changed: &[usize],
) {
    if changed.is_empty() {
        return;
    }

    let count = element.observe_children().into_iter().len();

    for child in children.element_iter().skip(count) {
        element.append(child);
    }

    for _ in children.element_count()..count {
        if let Some(child) = element.last_child() {
            element.remove(&child);
        }
    }

    if !changed.is_empty() {
        let children = children.element_iter().collect::<Vec<_>>();

        for &i in changed {
            let current: gtk4::Widget = element
                .observe_children()
                .into_iter()
                .nth(i)
                .unwrap()
                .unwrap()
                .downcast()
                .unwrap();

            element.insert_child_after(children[i], Some(&current));
            element.remove(&current);
        }
    }
}
