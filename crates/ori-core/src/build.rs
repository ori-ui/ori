use ori_reactive::{Callback, CallbackEmitter, OwnedSignal, Scope, Signal};

use crate::{Element, ElementView};

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

pub trait Bindings {
    type Setter<'a>
    where
        Self: 'a;

    fn setter(&mut self) -> Self::Setter<'_>;
}

pub trait Bindable<'a> {
    type Item: Send;

    fn bind(&mut self, cx: Scope, signal: Signal<Self::Item>);
}

impl<'a, T: Send + Sync + Clone + 'static> Bindable<'a> for OwnedSignal<T> {
    type Item = T;

    fn bind(&mut self, _cx: Scope, signal: Signal<Self::Item>) {
        self.bind(signal);
    }
}

pub trait Parent {
    type Child: ElementView;

    fn clear_children(&mut self);

    fn add_children(&mut self, child: impl Iterator<Item = Element<Self::Child>>) -> usize;

    fn set_children(&mut self, slot: usize, child: impl Iterator<Item = Element<Self::Child>>);
}
