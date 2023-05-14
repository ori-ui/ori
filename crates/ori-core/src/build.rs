use crate::{
    Callback, CallbackEmitter, EventSignal, IntoNode, Scope, SendSync, Sendable, SharedSignal,
    Signal, View,
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

pub trait BindCallback<'a> {
    type Event;

    fn bind(&mut self, cx: Scope<'a>, callback: impl FnMut(&Self::Event) + Sendable + 'a);
}

impl<'a, T: SendSync> BindCallback<'a> for EventSignal<T> {
    type Event = T;

    fn bind(&mut self, cx: Scope<'a>, callback: impl FnMut(&Self::Event) + Sendable + 'a) {
        self.subscribe(cx, callback);
    }
}

impl<'a, T: SendSync> BindCallback<'a> for Option<EventSignal<T>> {
    type Event = T;

    fn bind(&mut self, cx: Scope<'a>, callback: impl FnMut(&Self::Event) + Sendable + 'a) {
        let signal = self.get_or_insert_with(EventSignal::new);
        signal.subscribe(cx, callback);
    }
}

impl<'a, T: SendSync + 'a> BindCallback<'a> for CallbackEmitter<T> {
    type Event = T;

    fn bind(&mut self, cx: Scope<'a>, callback: impl FnMut(&Self::Event) + Sendable + 'a) {
        // SAFETY: `Callback` doesn't access any data allocated by `Scope::alloc_effect` in it's
        // Drop implementation.
        let callback = unsafe { cx.alloc_effect(Callback::new(callback)) };
        self.subscribe(callback);
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

    fn bind(&mut self, cx: Scope<'a>, signal: &'a Signal<Self::Item>);
}

impl<'a, T: Clone + PartialEq + SendSync + 'static> Bindable<'a> for &'a Signal<T> {
    type Item = T;

    fn bind(&mut self, cx: Scope<'a>, signal: &'a Signal<Self::Item>) {
        cx.bind(self, signal);
    }
}

impl<'a, T: Clone + PartialEq + SendSync + 'static> Bindable<'a> for SharedSignal<T> {
    type Item = T;

    fn bind(&mut self, cx: Scope<'a>, signal: &'a Signal<Self::Item>) {
        let this = cx.alloc(self.clone());
        cx.bind(this, signal);
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
