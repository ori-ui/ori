use std::{any::Any, future::Future, mem, sync::Arc};

use parking_lot::Mutex;

use crate::{
    context::Contexts, Callback, CallbackEmitter, Event, EventSink, OwnedSignal, ReadSignal,
    Resource, Runtime, ScopeId, Signal, Task,
};

use super::effect;

/// A reactive scope, that manages resources and contexts.
#[derive(Clone, Copy, Debug)]
pub struct Scope {
    pub(crate) id: ScopeId,
    pub(crate) contexts: Resource<Contexts>,
    pub(crate) event_sink: Resource<EventSink>,
    pub(crate) event_emitter: Resource<CallbackEmitter<Event>>,
}

impl Scope {
    /// Creates a new scope.
    ///
    /// # Arguments
    /// - `event_sink`: The event sink to use for this scope. This is where events will be sent
    /// when [`Scope::emit`] is called.
    /// - `event_emitter`: The callback emitter to use for this scope. When emitted callbacks
    /// registered with [`Scope::on_event`] and [`Scope::on`] will be called.
    pub fn new(event_sink: EventSink, event_emitter: CallbackEmitter<Event>) -> Self {
        let contexts = Resource::new_leaking(Contexts::new());
        let event_sink = Resource::new_leaking(event_sink);
        let event_emitter = Resource::new_leaking(event_emitter);

        let id = Runtime::global().create_scope(None);
        Runtime::global().manage_resource(id, contexts.id());
        Runtime::global().manage_resource(id, event_sink.id());
        Runtime::global().manage_resource(id, event_emitter.id());

        Self {
            id,
            contexts,
            event_sink,
            event_emitter,
        }
    }

    /// Runs `f` in a new scope that is immediately disposed when `f` returns.
    ///
    /// This is primarily used for testing.
    pub fn immediate<T>(f: impl FnOnce(Scope) -> T) -> T {
        let event_sink = EventSink::new(());
        let event_emitter = CallbackEmitter::new();

        let scope = Scope::new(event_sink, event_emitter);
        let result = f(scope);
        scope.dispose();

        result
    }

    /// Creates a child scope.
    ///
    /// A child scope is a scope that shares the same event sink and event emitter as its parent,
    /// and inherits its parent's contexts. When a child scope is dropped, its resources and contexts are
    /// dropped, but its parent's contexts are not. This is called internally in
    /// [`Scope::effect_scoped`].
    pub fn child(self) -> Scope {
        let contexts = self.contexts.get().unwrap_or_default();
        let contexts = Resource::new_leaking(contexts);

        let id = Runtime::global().create_scope(Some(self.id));
        Runtime::global().manage_resource(id, contexts.id());

        Scope {
            id,
            contexts,
            event_sink: self.event_sink,
            event_emitter: self.event_emitter,
        }
    }

    /// Runs a function without tracking any signals.
    pub fn untrack<T>(self, f: impl FnOnce() -> T) -> T {
        effect::untrack(f)
    }

    /// Creates a [`Resource`] that will be managed by this scope.
    pub fn resource<T: Send + Sync + 'static>(self, value: T) -> Resource<T> {
        let resource = Resource::new_leaking(value);
        self.manage_resource(resource);
        resource
    }

    /// Manages a [`Callback`], this will ensure that all [`WeakCallback`](crate::WeakCallback)s created from this
    /// `callback` will be able to upgrade.
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

    /// Manages a [`Resource`] in this scope.
    pub fn manage_resource<T: Send + 'static>(self, resource: Resource<T>) {
        Runtime::global().manage_resource(self.id, resource.id());
    }

    /// Manages a [`ReadSignal`] in this scope.
    pub fn manage_signal<T: Send + 'static>(self, signal: ReadSignal<T>) {
        Runtime::global().manage_resource(self.id, signal.resource.id());
        Runtime::global().manage_resource(self.id, signal.emitter.id());
    }

    /// Returns the [`EventSink`] for this scope.
    pub fn event_sink(self) -> EventSink {
        self.event_sink.get().expect("event sink was dropped")
    }

    /// Returns the [`CallbackEmitter`] for this scope.
    pub fn event_emitter(self) -> CallbackEmitter<Event> {
        (self.event_emitter.get()).expect("event callback emitter was dropped")
    }

    /// Emits an event.
    pub fn emit(self, event: impl Any + Send + Sync) {
        self.event_sink().emit(event);
    }

    /// Registers a callback to be called when an event is emitted.
    pub fn on_event(self, callback: impl FnMut(&Event) + Send + 'static) {
        let callback = Callback::new(callback);
        self.event_emitter().subscribe(&callback);
        self.manage_callback(callback);
    }

    /// Registers a callback to be called when an event of type `T` is emitted.
    pub fn on<T: Any + Send + Sync + 'static>(self, mut callback: impl FnMut(&T) + Send + 'static) {
        self.on_event(move |event| {
            if let Some(event) = event.get::<T>() {
                callback(event);
            }
        });
    }

    /// Pushes a context to this scope.
    pub fn with_context<C: Send + Sync + 'static>(self, context: C) -> Self {
        self.contexts.with_mut(|contexts| {
            contexts.push(context);
        });
        self
    }

    /// Returns `true` if this scope has a context of type `C`.
    pub fn has_context<C: Send + Sync + 'static>(self) -> bool {
        self.contexts
            .with(|contexts| contexts.contains::<C>())
            .unwrap_or(false)
    }

    /// Gets a context from this scope.
    pub fn get_context<C: Clone + Send + Sync + 'static>(self) -> Option<C> {
        let contexts = self.contexts.get()?;
        contexts.get()
    }

    /// Gets a context from this scope.
    ///
    /// # Panics
    /// - If the context is not found.
    #[track_caller]
    pub fn context<C: Clone + Send + Sync + 'static>(self) -> C {
        self.get_context().expect("context not found")
    }

    /// Gets a context wrapped in an [`Arc`] from this scope.
    ///
    /// This is useful for when `C` doesn't implement [`Clone`].
    pub fn get_context_arc<C: Send + Sync + 'static>(self) -> Option<Arc<C>> {
        let contexts = self.contexts.get()?;
        contexts.get_arc()
    }

    /// Gets a context wrapped in an [`Arc`] from this scope.
    ///
    /// # Panics
    /// - If the context is not found.
    #[track_caller]
    pub fn context_arc<C: Send + Sync + 'static>(self) -> Arc<C> {
        self.get_context_arc().expect("context not found")
    }

    /// Spawns a future on this scope.
    ///
    /// It will be polled once, when spawned, and when awakened will emit a [`Task`] to the
    /// [`EventSink`] of this scope.
    pub fn spawn(self, future: impl Future<Output = ()> + Send + 'static) {
        Task::spawn(self.event_sink(), future);
    }

    /// Creates a [`Signal`] that will be managed by this scope.
    pub fn signal<T: Send + Sync + 'static>(self, value: T) -> Signal<T> {
        let signal = Signal::new_leaking(value);

        self.manage_signal(*signal);

        signal
    }

    /// Creates an effect that will be rerun every time a dependency changes.
    #[track_caller]
    pub fn effect(self, effect: impl FnMut() + Send + 'static) {
        effect::create_effect(self.id, effect);
    }

    /// Creates an effect that will be rerun every time a dependency changes.
    ///
    /// This effect will take a child scope, which will be disposed when the effect is rerun.
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

    /// Creates a [`ReadSignal`] that is updated every time a dependency changes.
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

    /// Creates an [`OwnedSignal`] that is updated every time a dependency changes.
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

    /// Creates a [`ReadSignal`] that is updated every time a dependency changes.
    ///
    /// This signal will take a child scope, which will be disposed when the effect is rerun.
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

    /// Creates an [`OwnedSignal`] that is updated every time a dependency changes.
    ///
    /// This signal will take a child scope, which will be disposed when the effect is rerun.
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

    /// Dispose this scope.
    ///
    /// This will dispose all child [`Scope`]s, [`Resource`]s, [`Signal`]s, `effects` and
    /// `contexts`.
    #[track_caller]
    pub fn dispose(self) {
        Runtime::global().dispose_scope(self.id);
    }
}
