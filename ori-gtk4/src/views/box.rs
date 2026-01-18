use std::ops::{Deref, DerefMut};

use gtk4::prelude::{BoxExt as _, OrientableExt as _, WidgetExt as _};
use ori::{ElementSeq, Event};

use crate::{Context, views::Axis};

pub fn hbox<V>(contents: V) -> GtkBox<V> {
    GtkBox::new(Axis::Horizontal, contents)
}

pub fn vbox<V>(contents: V) -> GtkBox<V> {
    GtkBox::new(Axis::Vertical, contents)
}

#[must_use]
pub struct GtkBox<V> {
    contents: V,
    spacing:  u32,
    axis:     Axis,
}

impl<V> GtkBox<V> {
    pub fn new(axis: Axis, contents: V) -> Self {
        Self {
            contents,
            spacing: 0,
            axis,
        }
    }

    pub fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl<V> Deref for GtkBox<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.contents
    }
}

impl<V> DerefMut for GtkBox<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.contents
    }
}

impl<V> ori::ViewMarker for GtkBox<V> {}
impl<T, V> ori::View<Context, T> for GtkBox<V>
where
    V: ori::ViewSeq<Context, T, gtk4::Widget>,
{
    type Element = gtk4::Box;
    type State = (V::Elements, V::State);

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (mut children, state) = self.contents.seq_build(cx, data);

        let element = gtk4::Box::default();
        element.set_orientation(self.axis.into());
        element.set_spacing(self.spacing as i32);

        for child in children.iter_mut() {
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
        self.contents.seq_rebuild(
            children,
            state,
            cx,
            data,
            &mut old.contents,
        );

        update_children(element, children);

        // update state
        if self.spacing != old.spacing {
            element.set_spacing(self.spacing as i32);
        }

        if self.axis != old.axis {
            element.set_orientation(self.axis.into());
        }
    }

    fn teardown(
        &mut self,
        _element: Self::Element,
        (children, state): Self::State,
        cx: &mut Context,
    ) {
        self.contents.seq_teardown(children, state, cx);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (children, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut Event,
    ) -> ori::Action {
        let action = self.contents.seq_event(children, state, cx, data, event);

        update_children(element, children);

        action
    }
}

fn update_children(element: &gtk4::Box, children: &mut impl ElementSeq<gtk4::Widget>) {
    let mut prev = None::<&gtk4::Widget>;

    for child in children.iter_mut() {
        if prev.is_some_and(|p| p.next_sibling().as_ref() == Some(child)) {
            prev = Some(child);
            continue;
        }

        if super::is_parent(element, child) {
            element.reorder_child_after(child, prev);
        } else {
            element.insert_child_after(child, prev);
        }

        prev = Some(child);
    }

    let count = element.observe_children().into_iter().len();

    for _ in children.count()..count {
        if let Some(child) = element.last_child() {
            element.remove(&child);
        }
    }
}
