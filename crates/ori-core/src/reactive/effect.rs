use std::{cell::RefCell, fmt::Debug, ops::DerefMut, panic::Location, rc::Rc};

use crate::{Callback, Resource, Scope, Sendable, WeakCallbackEmitter};

thread_local! {
    static EFFECTS: RefCell<Vec<*mut EffectState<'static>>> = Default::default();
}

pub(crate) struct EffectState<'a> {
    location: &'static Location<'static>,
    callback: Callback<'a>,
    dependencies: Vec<WeakCallbackEmitter>,
}

impl<'a> EffectState<'a> {
    #[track_caller]
    fn empty() -> Self {
        Self {
            location: Location::caller(),
            callback: Callback::new(|()| {}),
            dependencies: Vec::new(),
        }
    }

    fn clear_dependencies(&mut self) {
        for dependency in &self.dependencies {
            if let Some(dependency) = dependency.upgrade() {
                dependency.unsubscribe(self.callback.as_ptr());
            }
        }

        self.dependencies.clear();
    }
}

impl<'a> Debug for EffectState<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EffectState")
            .field("location", &self.location)
            .finish()
    }
}

pub(crate) fn track_callback(callback: WeakCallbackEmitter) {
    EFFECTS.with(|effects| {
        if let Some(effect) = effects.borrow().last() {
            // SAFETY: effects is a thread local, so this is safe.
            let effect = unsafe { &mut **effect };
            effect.dependencies.push(callback);
        }
    });
}

pub(crate) fn untrack<T>(f: impl FnOnce() -> T) -> T {
    EFFECTS.with(|effects| {
        let tmp = effects.take();
        let result = f();
        effects.replace(tmp);
        result
    })
}

#[track_caller]
pub(crate) fn create_effect(cx: Scope, mut f: impl FnMut() + Sendable + 'static) {
    let caller = Location::caller();

    let effect = Resource::new_leaking(Rc::new(RefCell::new(EffectState::empty())));
    effect.manage(cx.id);

    let callback = Callback::new(move |()| {
        EFFECTS.with(|effects| {
            tracing::trace!("running effect at {}", caller);

            let len = effects.borrow().len();

            let effect = effect.get().unwrap();
            let mut effect = effect.borrow_mut();

            effect.clear_dependencies();

            effects.borrow_mut().push(effect.deref_mut() as *mut _);

            f();

            effects.borrow_mut().pop().expect("effects is empty");

            for emitter in &effect.dependencies {
                if let Some(emitter) = emitter.upgrade() {
                    emitter.subscribe(&effect.callback);
                }
            }

            debug_assert_eq!(effects.borrow().len(), len);
        });
    });

    effect.get().unwrap().borrow_mut().callback = callback.clone();

    callback.emit(&());
}
