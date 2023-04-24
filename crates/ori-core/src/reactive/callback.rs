use std::mem;

use crate::{Lock, Lockable, Sendable, Shared, Weak};

struct CallbackCollection<T> {
    callbacks: Vec<WeakCallback<T>>,
}

impl<T> Default for CallbackCollection<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> CallbackCollection<T> {
    fn new() -> Self {
        Self {
            callbacks: Vec::new(),
        }
    }

    fn len(&self) -> usize {
        self.callbacks.len()
    }

    fn contains(&self, ptr: CallbackPtr<T>) -> bool {
        for callback in &self.callbacks {
            if callback.callback.as_ptr() == ptr {
                return true;
            }
        }

        false
    }

    fn insert(&mut self, callback: WeakCallback<T>) {
        if !self.contains(callback.callback.as_ptr()) {
            self.callbacks.push(callback);
        }
    }

    fn remove(&mut self, ptr: CallbackPtr<T>) {
        let equals = |callback: &WeakCallback<T>| callback.callback.as_ptr() != ptr;
        self.callbacks.retain(equals);
    }
}

impl<T> IntoIterator for CallbackCollection<T> {
    type Item = WeakCallback<T>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.callbacks.into_iter()
    }
}

#[cfg(feature = "multithread")]
type RawCallback<'a, T> = dyn FnMut(&T) + Send + 'a;
#[cfg(not(feature = "multithread"))]
type RawCallback<'a, T> = dyn FnMut(&T) + 'a;

type CallbackPtr<T> = *const Lock<RawCallback<'static, T>>;
type Callbacks<T> = Lock<CallbackCollection<T>>;

/// A callback that can be called from any thread.
#[derive(Clone)]
pub struct Callback<'a, T = ()> {
    callback: Shared<Lock<RawCallback<'a, T>>>,
}

impl<'a, T> Callback<'a, T> {
    /// Creates a new callback.
    pub fn new(callback: impl FnMut(&T) + Sendable + 'a) -> Self {
        Self {
            callback: Shared::new(Lock::new(callback)),
        }
    }

    /// Downgrades the callback to a [`WeakCallback`].
    ///
    /// When the last strong reference is dropped, the callback will be dropped
    /// and all weak callbacks will be invalidated.
    pub fn downgrade(&self) -> WeakCallback<T> {
        // SAFETY: When the last strong reference is dropped, the callback will
        // be dropped and all weak callbacks will be invalidated. And can therefore
        // never be called. And since all strong references are tied to the lifetime
        // of the callback, it is safe to transmute the lifetime to static.
        let callback = unsafe {
            mem::transmute::<Weak<Lock<RawCallback<'a, T>>>, Weak<Lock<RawCallback<'static, T>>>>(
                Shared::downgrade(&self.callback),
            )
        };
        WeakCallback { callback }
    }

    /// Calls the callback.
    pub fn emit(&self, event: &T) {
        self.callback.lock_mut()(event);
    }
}

impl<'a, T> Default for Callback<'a, T> {
    fn default() -> Self {
        Callback::new(|_| {})
    }
}

/// A weak reference to a [`Callback`].
///
/// This is usually created by [`Callback::downgrade`].
#[derive(Clone)]
pub struct WeakCallback<T = ()> {
    callback: Weak<Lock<RawCallback<'static, T>>>,
}

impl<T> WeakCallback<T> {
    /// Creates a new weak callback from a weak reference.
    pub fn new(weak: Weak<Lock<RawCallback<'static, T>>>) -> Self {
        Self { callback: weak }
    }

    /// Tries to upgrade the weak callback to a [`Callback`].
    ///
    /// This will return `None` if all clones of the callback have been dropped.
    pub fn upgrade(&self) -> Option<Callback<T>> {
        Some(Callback {
            callback: self.callback.upgrade()?,
        })
    }

    /// Returns the raw pointer to the callback.
    pub fn as_ptr(&self) -> CallbackPtr<T> {
        self.callback.as_ptr() as CallbackPtr<T>
    }

    /// Tries to call the [`Callback`] if it is still alive.
    ///
    /// Returns `false` if it fails.
    pub fn emit(&self, event: &T) -> bool {
        if let Some(callback) = self.upgrade() {
            callback.emit(event);
        }

        self.callback.strong_count() > 0
    }
}

impl<T> Default for WeakCallback<T> {
    fn default() -> Self {
        // FIXME: this is a hack to get a valid pointer
        // but it just doesn't feel right
        Callback::default().downgrade()
    }
}

/// A [`Callback`] emitter.
///
/// This is used to store a list of callbacks and call them all.
/// All the callbacks are weak, so they must be kept alive by the user.
pub struct CallbackEmitter<T = ()> {
    callbacks: Shared<Callbacks<T>>,
}

impl<T> Default for CallbackEmitter<T> {
    fn default() -> Self {
        Self {
            callbacks: Shared::new(Lock::new(CallbackCollection::new())),
        }
    }
}

impl<T> Clone for CallbackEmitter<T> {
    fn clone(&self) -> Self {
        Self {
            callbacks: self.callbacks.clone(),
        }
    }
}

impl<T> CallbackEmitter<T> {
    /// Creates an empty callback emitter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of callbacks, valid or not.
    pub fn len(&self) -> usize {
        self.callbacks.lock_mut().len()
    }

    /// Returns `true` if there are no callbacks.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Downgrades the callback emitter to a [`WeakCallbackEmitter`].
    pub fn downgrade(&self) -> WeakCallbackEmitter<T> {
        WeakCallbackEmitter {
            callbacks: Shared::downgrade(&self.callbacks),
        }
    }

    /// Subscribes a callback to the emitter.
    ///
    /// The reference to the callback is weak, and will therefore not keep the
    /// callback alive. If the callback is dropped, it will be removed from the
    /// emitter.
    pub fn subscribe(&self, callback: &Callback<'_, T>) {
        self.subscribe_weak(callback.downgrade());
    }

    /// Subscribes a weak callback to the emitter.
    pub fn subscribe_weak(&self, callback: WeakCallback<T>) {
        self.callbacks.lock_mut().insert(callback);
    }

    /// Unsubscribes a callback from the emitter.
    pub fn unsubscribe(&self, ptr: CallbackPtr<T>) {
        self.callbacks.lock_mut().remove(ptr);
    }

    /// Clears all the callbacks, and calls them.
    pub fn emit(&self, event: &T) {
        let callbacks = mem::take(&mut *self.callbacks.lock_mut());

        for callback in callbacks.into_iter() {
            if let Some(callback) = callback.upgrade() {
                callback.emit(event);
            }
        }
    }
}

impl CallbackEmitter {
    /// Tracks `self` in the current `effect`.
    pub fn track(&self) {
        self.downgrade().track();
    }
}

/// A weak reference to a [`CallbackEmitter`].
///
/// This is usually created by [`CallbackEmitter::downgrade`].
pub struct WeakCallbackEmitter<T = ()> {
    callbacks: Weak<Callbacks<T>>,
}

impl<T> WeakCallbackEmitter<T> {
    /// Tries to upgrade the weak callback emitter to a [`CallbackEmitter`].
    pub fn upgrade(&self) -> Option<CallbackEmitter<T>> {
        Some(CallbackEmitter {
            callbacks: self.callbacks.upgrade()?,
        })
    }
}

impl<T> Clone for WeakCallbackEmitter<T> {
    fn clone(&self) -> Self {
        Self {
            callbacks: self.callbacks.clone(),
        }
    }
}

impl WeakCallbackEmitter {
    /// Tracks `self` in the current **effect**.
    pub fn track(&self) {
        super::effect::track_callback(self.clone());
    }
}
