use gtk4::{
    glib::{object::IsA, subclass::prelude::ObjectSubclassIsExt as _},
    prelude::{PopoverExt as _, WidgetExt as _},
};
use ori::Key;

use crate::{Context, View};

pub fn popover<V, P>(key: Key, content: V, popover: P) -> Popover<V, P> {
    Popover::new(key, content, popover)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PopoverCommand {
    Popup,
    Popdown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Position {
    Top,
    Right,
    Bottom,
    Left,
}

pub struct Popover<V, P> {
    key: Key,
    content: V,
    popover: P,
    autohide: bool,
    has_arrow: bool,
    position: Position,
}

impl<V, P> Popover<V, P> {
    pub fn new(key: Key, content: V, popover: P) -> Self {
        Self {
            key,
            content,
            popover,
            autohide: true,
            has_arrow: true,
            position: Position::Bottom,
        }
    }

    pub fn autohide(mut self, autohide: bool) -> Self {
        self.autohide = autohide;
        self
    }

    pub fn has_arrow(mut self, has_arrow: bool) -> Self {
        self.has_arrow = has_arrow;
        self
    }

    pub fn position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }
}

pub struct PopoverState<T, V, P>
where
    V: View<T>,
    P: View<T>,
{
    content_element: V::Element,
    content_state: V::State,
    popover_element: P::Element,
    popover_state: P::State,
    popover: gtk4::Popover,
}

impl<V, P> ori::ViewMarker for Popover<V, P> {}
impl<T, V, P> ori::View<Context, T> for Popover<V, P>
where
    V: View<T>,
    P: View<T>,
{
    type Element = PopoverReceiver;
    type State = PopoverState<T, V, P>;

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (content_element, content_state) = self.content.build(cx, data);
        let (popover_element, popover_state) = self.popover.build(cx, data);

        let popover = gtk4::Popover::new();
        popover.set_child(Some(&popover_element));

        popover.set_autohide(self.autohide);
        popover.set_has_arrow(self.has_arrow);
        popover.set_position(match self.position {
            Position::Top => gtk4::PositionType::Top,
            Position::Right => gtk4::PositionType::Right,
            Position::Bottom => gtk4::PositionType::Bottom,
            Position::Left => gtk4::PositionType::Left,
        });

        let element = PopoverReceiver::new();
        element.set_child(&content_element);
        element.set_popover(&popover);

        let state = PopoverState {
            content_element,
            content_state,
            popover_element,
            popover_state,
            popover,
        };

        (element, state)
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        old: &mut Self,
    ) -> bool {
        let content_changed = self.content.rebuild(
            &mut state.content_element,
            &mut state.content_state,
            cx,
            data,
            &mut old.content,
        );

        if content_changed {
            element.set_child(&state.content_element);
        }

        let popover_changed = self.popover.rebuild(
            &mut state.popover_element,
            &mut state.popover_state,
            cx,
            data,
            &mut old.popover,
        );

        if popover_changed {
            state.popover.set_child(Some(&state.popover_element));
        }

        if self.autohide != old.autohide {
            state.popover.set_autohide(self.autohide);
        }

        if self.has_arrow != old.has_arrow {
            state.popover.set_has_arrow(self.has_arrow);
        }

        if self.position != old.position {
            state.popover.set_position(match self.position {
                Position::Top => gtk4::PositionType::Top,
                Position::Right => gtk4::PositionType::Right,
                Position::Bottom => gtk4::PositionType::Bottom,
                Position::Left => gtk4::PositionType::Left,
            });
        }

        false
    }

    fn teardown(
        &mut self,
        _element: Self::Element,
        state: Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.content.teardown(
            state.content_element,
            state.content_state,
            cx,
            data,
        );

        self.popover.teardown(
            state.popover_element,
            state.popover_state,
            cx,
            data,
        );
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> (bool, ori::Action) {
        match event.take_targeted(self.key) {
            Some(PopoverCommand::Popup) => {
                state.popover.popup();
            }

            Some(PopoverCommand::Popdown) => {
                state.popover.popdown();
            }

            None => {}
        }

        let (content_changed, content_action) = self.content.event(
            &mut state.content_element,
            &mut state.content_state,
            cx,
            data,
            event,
        );

        if content_changed {
            element.set_child(&state.content_element);
        }

        let (popover_changed, popover_action) = self.popover.event(
            &mut state.popover_element,
            &mut state.popover_state,
            cx,
            data,
            event,
        );

        if popover_changed {
            state.popover.set_child(Some(&state.popover_element));
        }

        (false, content_action | popover_action)
    }
}

gtk4::glib::wrapper! {
    pub struct PopoverReceiver(
        ObjectSubclass<imp::PopoverReceiver>)
        @extends gtk4::Widget,
        @implements
            gtk4::Accessible,
            gtk4::Buildable,
            gtk4::ConstraintTarget;
}

impl Default for PopoverReceiver {
    fn default() -> Self {
        Self::new()
    }
}

impl PopoverReceiver {
    pub fn new() -> Self {
        gtk4::glib::Object::new()
    }

    pub fn set_child(&self, child: &impl IsA<gtk4::Widget>) {
        if let Some(child) = self
            .imp()
            .child
            .borrow_mut()
            .replace(child.as_ref().clone())
        {
            child.unparent();
        }

        child.as_ref().set_parent(self);
    }

    pub fn set_popover(&self, child: &impl IsA<gtk4::Popover>) {
        if let Some(child) = self
            .imp()
            .popover
            .borrow_mut()
            .replace(child.as_ref().clone())
        {
            child.unparent();
        }

        child.as_ref().set_parent(self);
    }
}

mod imp {
    use std::cell::RefCell;

    use gtk4::{glib, prelude::*, subclass::prelude::*};

    #[derive(Default)]
    pub struct PopoverReceiver {
        pub(super) child: RefCell<Option<gtk4::Widget>>,
        pub(super) popover: RefCell<Option<gtk4::Popover>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PopoverReceiver {
        const NAME: &'static str = "PopoverReceiver";
        type Type = super::PopoverReceiver;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for PopoverReceiver {
        fn dispose(&self) {
            if let Some(ref child) = *self.child.borrow() {
                child.unparent();
            }

            if let Some(ref popover) = *self.popover.borrow() {
                popover.unparent();
            }
        }
    }

    impl WidgetImpl for PopoverReceiver {
        fn measure(&self, orientation: gtk4::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            if let Some(ref child) = *self.child.borrow() {
                child.measure(orientation, for_size)
            } else {
                (0, 0, 0, 0)
            }
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            if let Some(ref child) = *self.child.borrow() {
                child.allocate(width, height, baseline, None);
            }

            if let Some(ref popover) = *self.popover.borrow() {
                popover.present();
            }
        }
    }
}
