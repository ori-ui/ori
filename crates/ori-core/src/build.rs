use crate::{Callback, CallbackEmitter, IntoNode, OwnedSignal, Scope, Signal, View};

pub trait Properties {
    type Setter<'a>
    where
        Self: 'a;

    fn setter(&mut self) -> Self::Setter<'_>;
}

pub trait Events {
    type Setter<'a>
    where
        Self: 'a;

    fn setter(&mut self) -> Self::Setter<'_>;
}

pub trait BindCallback {
    type Event;

    fn bind(&mut self, cx: Scope, callback: impl FnMut(&Self::Event) + 'static);
}

impl<T> BindCallback for CallbackEmitter<T> {
    type Event = T;

    fn bind(&mut self, cx: Scope, callback: impl FnMut(&Self::Event) + 'static) {
        let callback = Callback::new(callback);
        self.subscribe(&callback);
        cx.manage_callback(callback);
    }
}

pub trait Bindings {
    type Setter<'a>
    where
        Self: 'a;

    fn setter(&mut self) -> Self::Setter<'_>;
}

pub trait Bindable<'a> {
    type Item;

    fn bind(&mut self, cx: Scope, signal: Signal<Self::Item>);
}

impl<'a, T: Clone + PartialEq + 'static> Bindable<'a> for OwnedSignal<T> {
    type Item = T;

    fn bind(&mut self, _cx: Scope, signal: Signal<Self::Item>) {
        self.bind(signal);
    }
}

pub trait IntoChildren<I: IntoIterator> {
    fn into_children(self) -> I;
}

impl<T> IntoChildren<std::iter::Once<T>> for T {
    fn into_children(self) -> std::iter::Once<T> {
        std::iter::once(self)
    }
}

impl<T: IntoIterator> IntoChildren<T> for T {
    fn into_children(self) -> T {
        self
    }
}

pub trait Parent {
    type Child: View;

    fn add_child<I: IntoIterator, U: ?Sized>(&mut self, child: impl IntoChildren<I>)
    where
        I::Item: IntoNode<Self::Child, U>;

    fn with_child<I: IntoIterator, U: ?Sized>(mut self, child: impl IntoChildren<I>) -> Self
    where
        Self: Sized,
        I::Item: IntoNode<Self::Child, U>,
    {
        self.add_child(child);
        self
    }
}
