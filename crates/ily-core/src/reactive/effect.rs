use std::{cell::RefCell, mem, ops::DerefMut, panic::Location, rc::Rc};

use crate::{Scope, WeakCallback, WeakCallbackEmitter};

thread_local! {
    static EFFECTS: RefCell<Vec<*mut EffectState<'static>>> = Default::default();
}

struct EffectState<'a> {
    location: &'static Location<'static>,
    callback: Rc<RefCell<dyn FnMut() + 'a>>,
    dependencies: Vec<WeakCallbackEmitter>,
}

impl<'a> EffectState<'a> {
    #[track_caller]
    fn empty() -> Self {
        Self {
            location: Location::caller(),
            callback: Rc::new(RefCell::new(|| {})),
            dependencies: Vec::new(),
        }
    }

    fn clear_dependencies(&mut self) {
        for dependency in &self.dependencies {
            if let Some(dependency) = dependency.upgrade() {
                let ptr = Rc::as_ptr(&self.callback);
                dependency.unsubscribe(unsafe { mem::transmute(ptr) });
            }
        }

        self.dependencies.clear();
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
pub(crate) fn create_effect<'a>(cx: Scope<'a>, mut f: impl FnMut() + 'a) {
    // SAFETY: `Effect` is `!Drop`, so it's safe to use `alloc_unsafe`.
    let effect = unsafe { cx.alloc_unsafe(RefCell::new(EffectState::empty())) };

    let callback = Rc::new(RefCell::new(move || {
        EFFECTS.with(|effects| {
            let len = effects.borrow().len();

            let mut effect = effect.borrow_mut();
            let effect_ptr = effect.deref_mut() as *mut EffectState<'a>;
            let static_effect_ptr = effect_ptr.cast::<EffectState<'static>>();

            // clear the dependencies of `effect` so that we only track the new ones
            effect.clear_dependencies();

            // push the effect onto the stack so that it can be used by `track_callback`
            effects.borrow_mut().push(static_effect_ptr);

            // drop lock the scope to ensure that child scopes stay alive for the duration of the effect
            cx.drop_lock();
            // now we can run the effect
            tracing::trace!("running effect at {}", effect.location);
            f();
            // we release the lock so that child scopes can be dropped
            cx.release_drop_lock();

            // pop the effect from the stack
            effects.borrow_mut().pop().expect("effect stack underflow");

            // subscribe to all dependencies
            for emitter in &effect.dependencies {
                if let Some(emitter) = emitter.upgrade() {
                    let callback = unsafe { mem::transmute(&effect.callback) };
                    let callback = WeakCallback::new(Rc::downgrade(callback));
                    emitter.subscribe_weak(callback);
                }
            }

            debug_assert_eq!(len, effects.borrow().len());
        })
    }));

    effect.borrow_mut().callback = callback.clone();

    callback.borrow_mut()();
}
