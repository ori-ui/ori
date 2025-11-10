use std::ops::{Deref, DerefMut};

use gtk4::prelude::{BoxExt as _, OrientableExt as _, WidgetExt as _};
use ori::Event;

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
    type State = (Vec<gtk4::Widget>, V::SeqState);

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (children, state) = self.content.seq_build(cx, data);

        let element = gtk4::Box::default();
        element.set_orientation(self.axis.into());
        element.set_spacing(self.spacing as i32);

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
    ) -> bool {
        let changed = self.content.seq_rebuild(
            children,
            state,
            cx,
            data,
            &mut old.content,
        );

        if changed {
            update_children(element, children);
        }

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

        if changed {
            update_children(element, children);
        }

        (false, action)
    }
}

fn update_children(element: &gtk4::Box, children: &[gtk4::Widget]) {
    // add new children
    for child in children.iter() {
        if !super::is_parent(element, child) {
            element.append(child);
        }
    }

    // reorder children
    for child in children.iter().rev() {
        if super::is_parent(element, child) {
            element.reorder_child_after(child, None::<&gtk4::Widget>);
        }
    }

    // remove children
    let count = element.observe_children().into_iter().len();

    for _ in children.len()..count {
        if let Some(child) = element.last_child() {
            element.remove(&child);
        }
    }
}
