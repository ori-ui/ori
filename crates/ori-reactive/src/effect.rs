//! Effects are closures that are re-run when their dependencies change.
//!
//! Here is a list of common dependencies:
//! - [`Signal`](crate::Signal)
//! - [`ReadSignal`](crate::ReadSignal)
//! - [`OwnedSignal`](crate::OwnedSignal)
//! - [`Atom`](crate::Atom)
//! - [`AtomRef`](crate::AtomRef)
//!
//! # Example
//! ```
//! # use ori_reactive::prelude::*;
//! # Scope::immediate(|cx| {
//! // create a signal
//! let counter = cx.signal(1);
//! // when the effect is run once when created
//! cx.effect(move || *counter.modify() *= 2);
//!
//! assert_eq!(counter.get(), 2);
//!
//! // and again when `counter` is modified
//! counter.set(4);
//! assert_eq!(counter.get(), 8);
//! });
//! ```

use std::{cell::RefCell, fmt::Debug, ops::DerefMut, panic::Location, sync::Arc};

use parking_lot::Mutex;

use crate::{Callback, Resource, ScopeId, WeakCallbackEmitter};

thread_local! {
    static EFFECT_STACK: RefCell<Vec<*mut EffectState>> = Default::default();
    static CAPTURED_EFFECTS: RefCell<Option<Vec<Callback<'static, ()>>>> = Default::default();
}

pub(crate) struct EffectState {
    location: &'static Location<'static>,
    callback: Callback<'static>,
    dependencies: Vec<WeakCallbackEmitter>,
}

impl EffectState {
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

impl Debug for EffectState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EffectState")
            .field("location", &self.location)
            .finish()
    }
}

/// Subscribes the last effect on the effect stack to the given `emitter`.
pub fn track_callback(emitter: WeakCallbackEmitter) {
    EFFECT_STACK.with(|effects| {
        if let Some(effect) = effects.borrow().last() {
            // SAFETY: EFFECT_STACK is thread local, and no other mutable references to `effect`
            // can be created while this function is running.
            let effect = unsafe { &mut **effect };
            effect.dependencies.push(emitter);
        }
    });
}

/// Runs `f` ignoring all calls to [`track_callback`].
pub fn untrack<T>(f: impl FnOnce() -> T) -> T {
    EFFECT_STACK.with(|effects| {
        let tmp = effects.take();
        let result = f();
        effects.replace(tmp);
        result
    })
}

fn set_capture(value: Option<Vec<Callback<'static, ()>>>) -> Option<Vec<Callback<'static, ()>>> {
    CAPTURED_EFFECTS.with(|capture| {
        let mut capture = capture.borrow_mut();
        let tmp = capture.take();
        *capture = value;
        tmp
    })
}

/// Captures all effects that are called during the execution of `f`.
#[must_use]
pub fn capture_effects(f: impl FnOnce()) -> Vec<Callback<'static, ()>> {
    let tmp = set_capture(Some(Vec::new()));
    f();
    set_capture(tmp).unwrap()
}

/// Runs `f` and captures all effects that are called during the execution of `f`.
/// After `f` is done, all captured effects are emitted.
///
/// This is useful when an effect might try to lock a resource that is already locked.
pub fn delay_effects(f: impl FnOnce()) {
    let effects = capture_effects(f);

    for effect in effects {
        effect.emit(&());
    }
}

/// Creates a new effect. The effect is managed by the given `scope`, and when the scope is
/// disposed, the effect is disposed as well, meaning it can no longer run.
///
/// For more information, see the [module-level documentation](crate::effect).
#[track_caller]
pub fn create_effect(scope: ScopeId, mut f: impl FnMut() + Send + 'static) {
    let caller = Location::caller();

    let effect = Resource::new_leaking(Arc::new(Mutex::new(EffectState::empty())));
    effect.manage(scope);

    let callback = Callback::new(move |()| {
        let mut captured = false;

        CAPTURED_EFFECTS.with(|capture| {
            let mut capture = capture.borrow_mut();

            if let Some(capture) = capture.as_mut() {
                let callback = effect.get().unwrap().lock().callback.clone();
                capture.push(callback);
                captured = true;
            }
        });

        if captured {
            return;
        }

        EFFECT_STACK.with(|effects| {
            tracing::trace!("running effect at {}", caller);

            let len = effects.borrow().len();

            let effect = effect.get().unwrap();
            let mut effect = effect.lock();

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

    effect.get().unwrap().lock().callback = callback.clone();

    callback.emit(&());
}
