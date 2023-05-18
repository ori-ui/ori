use std::{future::Future, mem};

use crate::{
    Callback, EventSink, Lock, Lockable, OwnedSignal, ReadSignal, Resource, Runtime, ScopeId,
    Sendable, Shared, Signal, Task,
};

use super::effect;

#[derive(Clone, Copy, Debug)]
pub struct Scope {
    pub(crate) id: ScopeId,
    pub(crate) event_sink: Resource<EventSink>,
}

impl Scope {
    pub fn new(event_sink: EventSink) -> Self {
        let event_sink = Resource::new_leaking(event_sink);

        Runtime::with_global_runtime(|runtime| {
            let id = runtime.create_scope(None);
            runtime.manage_resource(id, event_sink.id());

            Self { id, event_sink }
        })
    }

    pub fn child(self) -> Scope {
        Runtime::with_global_runtime(|runtime| {
            let id = runtime.create_scope(Some(self.id));

            Scope {
                id,
                event_sink: self.event_sink,
            }
        })
    }

    pub fn untrack<T>(self, f: impl FnOnce() -> T) -> T {
        effect::untrack(f)
    }

    pub fn resource<T: Sendable + 'static>(self, value: T) -> Resource<T> {
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

    pub fn manage_resource<T: Sendable + 'static>(self, resource: Resource<T>) {
        Runtime::with_global_runtime(|runtime| {
            runtime.manage_resource(self.id, resource.id());
        })
    }

    pub fn manage_signal<T: Sendable + 'static>(self, signal: Signal<T>) {
        Runtime::with_global_runtime(|runtime| {
            runtime.manage_resource(self.id, signal.resource.id());
            runtime.manage_resource(self.id, signal.emitter.id());
        })
    }

    pub fn event_sink(&self) -> EventSink {
        self.event_sink.get().expect("event sink was dropped")
    }

    pub fn spawn(self, future: impl Future<Output = ()> + Send + 'static) {
        Task::spawn(self.event_sink(), future);
    }

    pub fn signal<T: Sendable + 'static>(self, value: T) -> Signal<T> {
        let signal = Signal::new_leaking(value);

        self.manage_signal(signal);

        signal
    }

    #[track_caller]
    pub fn effect(self, effect: impl FnMut() + Sendable + 'static) {
        effect::create_effect(self, effect);
    }

    #[track_caller]
    pub fn effect_scoped(self, mut effect: impl FnMut(Scope) + Sendable + 'static) {
        let mut scope = None::<Scope>;
        self.effect(move || {
            if let Some(scope) = scope {
                scope.dispose();
            }

            let child = self.child();
            effect(child);
            scope = Some(child);
        });
    }

    #[track_caller]
    pub fn memo<T: Sendable>(
        self,
        mut memo: impl FnMut() -> T + Sendable + 'static,
    ) -> ReadSignal<T> {
        let signal = Shared::new(Lock::new(None::<Signal<T>>));

        self.effect({
            let signal = signal.clone();

            move || {
                let value = memo();

                if signal.lock_mut().is_some() {
                    signal.lock_mut().unwrap().set(value);
                } else {
                    *signal.lock_mut() = Some(self.signal(value));
                }
            }
        });

        let signal = signal.lock_mut();
        **signal.as_ref().unwrap()
    }

    #[track_caller]
    pub fn owned_memo<T: Sendable>(
        self,
        mut memo: impl FnMut() -> T + Sendable + 'static,
    ) -> OwnedSignal<T> {
        let signal = Shared::new(Lock::new(None::<OwnedSignal<T>>));

        self.effect({
            let signal = signal.clone();

            move || {
                let value = memo();

                if signal.lock_mut().is_some() {
                    signal.lock_mut().as_ref().unwrap().set(value);
                } else {
                    *signal.lock_mut() = Some(OwnedSignal::new(value));
                }
            }
        });

        let signal = signal.lock_mut();
        signal.as_ref().unwrap().clone()
    }

    #[track_caller]
    pub fn memo_scoped<T: Sendable>(
        self,
        mut memo: impl FnMut(Scope) -> T + Sendable + 'static,
    ) -> ReadSignal<T> {
        let signal = Shared::new(Lock::new(None::<Signal<T>>));

        self.effect_scoped({
            let signal = signal.clone();

            move |child| {
                let value = memo(child);

                if signal.lock_mut().is_some() {
                    signal.lock_mut().unwrap().set(value);
                } else {
                    *signal.lock_mut() = Some(child.signal(value));
                }
            }
        });

        let signal = signal.lock_mut();
        **signal.as_ref().unwrap()
    }

    #[track_caller]
    pub fn owned_memo_scoped<T: Sendable>(
        self,
        mut memo: impl FnMut(Scope) -> T + Sendable + 'static,
    ) -> OwnedSignal<T> {
        let signal = Shared::new(Lock::new(None::<OwnedSignal<T>>));

        self.effect_scoped({
            let signal = signal.clone();

            move |child| {
                let value = memo(child);

                if signal.lock_mut().is_some() {
                    signal.lock_mut().as_ref().unwrap().set(value);
                } else {
                    *signal.lock_mut() = Some(OwnedSignal::new(value));
                }
            }
        });

        let signal = signal.lock_mut();
        signal.as_ref().unwrap().clone()
    }

    pub fn dispose(self) {
        Runtime::with_global_runtime(|runtime| {
            runtime.dispose_scope(self.id);
        });
    }
}
