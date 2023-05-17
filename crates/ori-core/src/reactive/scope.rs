use std::{cell::Cell, panic::Location, rc::Rc};

use crate::{Lock, Lockable, OwnedSignal, Resource, Runtime, ScopeId, Shared, Signal};

use super::effect;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Scope {
    pub(crate) id: ScopeId,
}

impl Scope {
    fn from_raw(id: ScopeId) -> Self {
        Self { id }
    }

    pub fn new() -> Self {
        Runtime::with_global_runtime(|runtime| {
            let id = runtime.create_scope(None);
            Self { id }
        })
    }

    pub fn child(self) -> Scope {
        Runtime::with_global_runtime(|runtime| {
            let id = runtime.create_scope(Some(self.id));
            Scope { id }
        })
    }

    pub fn untrack<T>(self, f: impl FnOnce() -> T) -> T {
        effect::untrack(f)
    }

    pub fn resource<T: 'static>(self, value: T) -> Resource<T> {
        let resource = Resource::new_leaking(value);
        self.manage_resource(resource);
        resource
    }

    pub fn manage_resource<T: 'static>(self, resource: Resource<T>) {
        Runtime::with_global_runtime(|runtime| {
            runtime.manage_resource(self.id, resource.id());
        })
    }

    pub fn manage_signal<T: 'static>(self, signal: Signal<T>) {
        Runtime::with_global_runtime(|runtime| {
            runtime.manage_resource(self.id, signal.resource.id());
            runtime.manage_resource(self.id, signal.emitter.id());
        })
    }

    pub fn signal<T: 'static>(self, value: T) -> Signal<T> {
        let signal = Signal::new_leaking(value);

        self.manage_signal(signal);

        signal
    }

    #[track_caller]
    pub fn effect(self, effect: impl FnMut() + 'static) {
        effect::create_effect(self, effect);
    }

    #[track_caller]
    pub fn effect_scoped(self, mut effect: impl FnMut(Scope) + 'static) {
        let caller = Location::caller();
        let mut scope = None::<ScopeId>;
        let id = self.id;
        Scope::from_raw(id).effect(move || {
            tracing::trace!("effect created at {}", caller);

            if let Some(scope) = scope {
                Scope::from_raw(scope).dispose();
            }

            let child = Scope::from_raw(id).child();
            effect(child);
            scope = Some(child.id);
        });
    }

    #[track_caller]
    pub fn memo<T>(self, mut memo: impl FnMut() -> T + 'static) -> Signal<T> {
        let signal = Rc::new(Cell::new(None::<Signal<T>>));

        self.effect({
            let signal = signal.clone();

            move || {
                let value = memo();

                if signal.get().is_some() {
                    signal.get().unwrap().set(value);
                } else {
                    signal.set(Some(self.signal(value)));
                }
            }
        });

        signal.get().unwrap()
    }

    #[track_caller]
    pub fn owned_memo<T>(self, mut memo: impl FnMut() -> T + 'static) -> OwnedSignal<T> {
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
    pub fn memo_scoped<T>(self, mut memo: impl FnMut(Scope) -> T + 'static) -> Signal<T> {
        let signal = Rc::new(Cell::new(None::<Signal<T>>));

        self.effect_scoped({
            let signal = signal.clone();

            move |child| {
                let value = memo(child);

                if signal.get().is_some() {
                    signal.get().unwrap().set(value);
                } else {
                    signal.set(Some(child.signal(value)));
                }
            }
        });

        signal.get().unwrap()
    }

    #[track_caller]
    pub fn owned_memo_scoped<T>(
        self,
        mut memo: impl FnMut(Scope) -> T + 'static,
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

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn effect() {
        let scope = Scope::new();
        let signal = scope.memo(|| 0);
        assert_eq!(signal.get(), 0);
        scope.effect(move || {
            signal.set(1);
        });
        assert_eq!(signal.get(), 1);
        scope.dispose();
    }

    #[test]
    fn effect_scoped() {
        let scope = Scope::new();
        let signal = scope.signal(0);
        assert_eq!(signal.get(), 0);
        scope.effect_scoped(move |child| {
            child.effect(move || {
                signal.set(2);
            });
        });
        assert_eq!(signal.get(), 2);
        scope.dispose();
    }
}
