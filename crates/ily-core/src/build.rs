use crate::{EventSignal, Scope, SendSync, Sendable, SharedSignal, Signal, View};

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

    fn bind<'a>(&mut self, cx: Scope<'a>, callback: impl FnMut(&Self::Event) + Sendable + 'a);
}

impl<T: SendSync> BindCallback for EventSignal<T> {
    type Event = T;

    fn bind<'a>(&mut self, cx: Scope<'a>, callback: impl FnMut(&Self::Event) + Sendable + 'a) {
        self.subscribe(cx, callback);
    }
}

impl<T: SendSync> BindCallback for Option<EventSignal<T>> {
    type Event = T;

    fn bind<'a>(&mut self, cx: Scope<'a>, callback: impl FnMut(&Self::Event) + Sendable + 'a) {
        let signal = self.get_or_insert_with(EventSignal::new);
        signal.subscribe(cx, callback);
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
    fn add_child(&mut self, child: impl View);
}
