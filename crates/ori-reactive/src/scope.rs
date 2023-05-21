use std::{any::Any, future::Future, mem, sync::Arc};

use parking_lot::Mutex;

use crate::{
    Callback, CallbackEmitter, Event, EventSink, OwnedSignal, ReadSignal, Resource, Runtime,
    ScopeId, Signal, Task,
};

use super::effect;

#[derive(Clone, Copy, Debug)]
pub struct Scope {
    pub(crate) id: ScopeId,
    pub(crate) event_sink: Resource<EventSink>,
    pub(crate) event_callbacks: Resource<CallbackEmitter<Event>>,
}

impl Scope {
    pub fn new(event_sink: EventSink, event_callback: CallbackEmitter<Event>) -> Self {
        let event_sink = Resource::new_leaking(event_sink);
        let event_callbacks = Resource::new_leaking(event_callback);

        let id = Runtime::global().create_scope(None);
        Runtime::global().manage_resource(id, event_sink.id());
        Runtime::global().manage_resource(id, event_callbacks.id());

        Self {
            id,
            event_sink,
            event_callbacks,
        }
    }

    pub fn child(self) -> Scope {
        let id = Runtime::global().create_scope(Some(self.id));

        Scope {
            id,
            event_sink: self.event_sink,
            event_callbacks: self.event_callbacks,
        }
    }

    pub fn untrack<T>(self, f: impl FnOnce() -> T) -> T {
        effect::untrack(f)
    }

    pub fn resource<T: Send + Sync + 'static>(self, value: T) -> Resource<T> {
        let resource = Resource::new_leaking(value);
        self.manage_resource(resource);
        resource
    }

    pub fn manage_callback<T>(self, callback: Callback<'static, T>) {
        // do not think about this too much, it will drive you mad
        unsafe {
            // SAFETY: the transmuted callback just exists to keep the callback alive
            // and allow WeakCallbacks to upgrade, this transmutation is needed because
            // T might not be 'static, but we know that the transmuted callback will never be
            // used, i hope
            let callback = mem::transmute::<Callback<'static, T>, Callback<'static, ()>>(callback);
            self.resource(callback);
        };
    }

    pub fn manage_resource<T: Send + 'static>(self, resource: Resource<T>) {
        Runtime::global().manage_resource(self.id, resource.id());
    }

    pub fn manage_signal<T: Send + 'static>(self, signal: Signal<T>) {
        Runtime::global().manage_resource(self.id, signal.resource.id());
        Runtime::global().manage_resource(self.id, signal.emitter.id());
    }

    pub fn event_sink(self) -> EventSink {
        self.event_sink.get().expect("event sink was dropped")
    }

    pub fn event_callbacks(self) -> CallbackEmitter<Event> {
        (self.event_callbacks.get()).expect("event callback emitter was dropped")
    }

    pub fn emit_event(self, event: impl Any + Send + Sync) {
        self.event_sink().emit(event);
    }

    pub fn on_event(self, callback: impl FnMut(&Event) + Send + 'static) {
        let callback = Callback::new(callback);
        self.event_callbacks().subscribe(&callback);
        self.manage_callback(callback);
    }

    pub fn spawn(self, future: impl Future<Output = ()> + Send + 'static) {
        Task::spawn(self.event_sink(), future);
    }

    pub fn signal<T: Send + Sync + 'static>(self, value: T) -> Signal<T> {
        let signal = Signal::new_leaking(value);

        self.manage_signal(signal);

        signal
    }

    #[track_caller]
    pub fn effect(self, effect: impl FnMut() + Send + 'static) {
        effect::create_effect(self, effect);
    }

    #[track_caller]
    pub fn effect_scoped(self, mut effect: impl FnMut(Scope) + Send + 'static) {
        let mut scope = None::<Scope>;
        self.effect(move || {
            if let Some(scope) = scope.take() {
                scope.dispose();
            }

            let child = self.child();
            effect(child);
            scope = Some(child);
        });
    }

    #[track_caller]
    pub fn memo<T: Send + Sync>(
        self,
        mut memo: impl FnMut() -> T + Send + 'static,
    ) -> ReadSignal<T> {
        let signal = Arc::new(Mutex::new(None::<Signal<T>>));

        self.effect({
            let signal = signal.clone();

            move || {
                let value = memo();

                if signal.lock().is_some() {
                    signal.lock().unwrap().set(value);
                } else {
                    *signal.lock() = Some(self.signal(value));
                }
            }
        });

        let signal = signal.lock();
        **signal.as_ref().unwrap()
    }

    #[track_caller]
    pub fn owned_memo<T: Send + Sync>(
        self,
        mut memo: impl FnMut() -> T + Send + 'static,
    ) -> OwnedSignal<T> {
        let signal = Arc::new(Mutex::new(None::<OwnedSignal<T>>));

        self.effect({
            let signal = signal.clone();

            move || {
                let value = memo();

                if signal.lock().is_some() {
                    signal.lock().as_ref().unwrap().set(value);
                } else {
                    *signal.lock() = Some(OwnedSignal::new(value));
                }
            }
        });

        let signal = signal.lock();
        signal.as_ref().unwrap().clone()
    }

    #[track_caller]
    pub fn memo_scoped<T: Send + Sync>(
        self,
        mut memo: impl FnMut(Scope) -> T + Send + 'static,
    ) -> ReadSignal<T> {
        let signal = Arc::new(Mutex::new(None::<Signal<T>>));

        self.effect_scoped({
            let signal = signal.clone();

            move |child| {
                let value = memo(child);

                if signal.lock().is_some() {
                    signal.lock().unwrap().set(value);
                } else {
                    *signal.lock() = Some(child.signal(value));
                }
            }
        });

        let signal = signal.lock();
        **signal.as_ref().unwrap()
    }

    #[track_caller]
    pub fn owned_memo_scoped<T: Send + Sync>(
        self,
        mut memo: impl FnMut(Scope) -> T + Send + 'static,
    ) -> OwnedSignal<T> {
        let signal = Arc::new(Mutex::new(None::<OwnedSignal<T>>));

        self.effect_scoped({
            let signal = signal.clone();

            move |child| {
                let value = memo(child);

                if signal.lock().is_some() {
                    signal.lock().as_ref().unwrap().set(value);
                } else {
                    *signal.lock() = Some(OwnedSignal::new(value));
                }
            }
        });

        let signal = signal.lock();
        signal.as_ref().unwrap().clone()
    }

    pub fn dispose(self) {
        Runtime::global().dispose_scope(self.id);
    }
}
