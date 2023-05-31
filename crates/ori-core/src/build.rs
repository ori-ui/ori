use ori_reactive::{Callback, CallbackEmitter, OwnedSignal, Scope, Signal};

use crate::{ElementView, IntoElement, Node};

/// A trait for setting properties on an element.
pub trait Properties {
    /// The setter type.
    type Setter<'a>
    where
        Self: 'a;

    /// Returns [`Self::Setter`].
    fn setter(&mut self) -> Self::Setter<'_>;
}

/// A trait for setting events on an element.
pub trait Events {
    /// The setter type.
    type Setter<'a>
    where
        Self: 'a;

    /// Returns [`Self::Setter`].
    fn setter(&mut self) -> Self::Setter<'_>;
}

/// A trait that is implemented for every type a callback can be subscribed to.
pub trait BindCallback {
    /// The event type.
    type Event;

    /// Binds a callback to the signal.
    fn bind(&mut self, cx: Scope, callback: impl FnMut(&Self::Event) + Send + 'static);
}

impl<T> BindCallback for CallbackEmitter<T> {
    type Event = T;

    fn bind(&mut self, cx: Scope, callback: impl FnMut(&Self::Event) + Send + 'static) {
        let callback = Callback::new(callback);
        self.subscribe(&callback);
        cx.manage_callback(callback);
    }
}

/// A trait for setting bindings on an element.
pub trait Bindings {
    /// The setter type.
    type Setter<'a>
    where
        Self: 'a;

    /// Returns [`Self::Setter`].
    fn setter(&mut self) -> Self::Setter<'_>;
}

/// A trait implemented for every type that can be bound to a signal.
pub trait Bindable<'a> {
    /// The item type.
    type Item: Send;

    /// Binds the signal to the value.
    fn bind(&mut self, cx: Scope, signal: Signal<Self::Item>);
}

impl<'a, T: Send + Sync + Clone + 'static> Bindable<'a> for OwnedSignal<T> {
    type Item = T;

    fn bind(&mut self, _cx: Scope, signal: Signal<Self::Item>) {
        self.bind(signal);
    }
}

/// A trait for setting children on an element.
pub trait Parent {
    /// The child type.
    type Child: ElementView;

    /// Clears all children.
    fn clear_children(&mut self);

    /// Adds `children` to a new slot and returns the slot index.
    fn add_children(&mut self, children: impl Iterator<Item = Node<Self::Child>>) -> usize;

    /// Sets the children of `slot` to `children`.
    fn set_children(&mut self, slot: usize, children: impl Iterator<Item = Node<Self::Child>>);

    /// Adds `child` to a new slot and returns the slot index.
    fn add_child(&mut self, child: impl IntoElement<Self::Child>) -> usize {
        self.add_children(std::iter::once(Node::element(child.into_element())))
    }

    /// Adds `children` to a new slot.
    fn with_children(mut self, children: impl Iterator<Item = Node<Self::Child>>) -> Self
    where
        Self: Sized,
    {
        self.add_children(children);
        self
    }

    /// Adds `child` to a new slot.
    fn with_child(mut self, child: impl IntoElement<Self::Child>) -> Self
    where
        Self: Sized,
    {
        self.add_child(child);
        self
    }
}
