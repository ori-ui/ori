use crate::{Scope, SharedSignal};

pub struct EventSignal<T: 'static> {
    signal: SharedSignal<Option<T>>,
}

impl<T> EventSignal<T> {
    pub fn new() -> Self {
        Self {
            signal: SharedSignal::new(None),
        }
    }

    pub fn track(&self) {
        self.signal.track();
    }

    pub fn subscribe<'a>(&self, cx: Scope<'a>, mut callback: impl FnMut(&T) + 'a) {
        let signal = self.signal.clone();
        cx.effect(move || {
            if let Some(event) = signal.get().as_ref() {
                callback(event);
            }
        });
    }

    pub fn emit(&self, event: T) {
        self.signal.set(Some(event));
    }
}

impl<T> Default for EventSignal<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for EventSignal<T> {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal.clone(),
        }
    }
}
