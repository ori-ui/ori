use crate::{
    Callback, CallbackEmitter, IntoNode, OwnedSignal, Scope, SendSync, Sendable, Signal, View,
};

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

    fn bind(&mut self, cx: Scope, callback: impl FnMut(&Self::Event) + Sendable + 'static);
}

impl<T: SendSync + 'static> BindCallback for CallbackEmitter<T> {
    type Event = T;

    fn bind(&mut self, cx: Scope, callback: impl FnMut(&Self::Event) + Sendable + 'static) {
        let callback = Callback::new(callback);
        self.subscribe(&callback);
        cx.resource(callback);
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

impl<'a, T: Clone + PartialEq + SendSync + 'static> Bindable<'a> for OwnedSignal<T> {
    type Item = T;

    fn bind(&mut self, _cx: Scope, signal: Signal<Self::Item>) {
        self.bind(signal);
    }
}

pub trait Parent {
    type Child: View;

    fn add_child<U: ?Sized>(&mut self, child: impl IntoNode<Self::Child, U>);

    fn with_child<U: ?Sized>(mut self, child: impl IntoNode<Self::Child, U>) -> Self
    where
        Self: Sized,
    {
        self.add_child(child);
        self
    }
}
