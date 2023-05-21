use std::{
    fmt::Debug,
    mem,
    sync::{Arc, Weak},
};

use parking_lot::Mutex;

use crate::{Callback, WeakCallback};

use super::CallbackPtr;

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
            if callback.as_ptr() == ptr {
                return true;
            }
        }

        false
    }

    fn insert(&mut self, callback: WeakCallback<T>) {
        if !self.contains(callback.as_ptr()) {
            self.callbacks.push(callback);
        }
    }

    fn remove(&mut self, ptr: CallbackPtr<T>) {
        let equals = |callback: &WeakCallback<T>| callback.as_ptr() != ptr;
        self.callbacks.retain(equals);
    }

    fn iter(&self) -> impl Iterator<Item = &WeakCallback<T>> {
        self.callbacks.iter()
    }
}

impl<T> IntoIterator for CallbackCollection<T> {
    type Item = WeakCallback<T>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.callbacks.into_iter()
    }
}

impl<T> Debug for CallbackCollection<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallbackCollection")
            .field("callbacks", &self.callbacks)
            .finish()
    }
}

type Callbacks<T> = Mutex<CallbackCollection<T>>;

/// A [`Callback`] emitter.
///
/// This is used to store a list of callbacks and call them all.
/// All the callbacks are weak, so they must be kept alive by the user.
pub struct CallbackEmitter<T = ()> {
    callbacks: Arc<Callbacks<T>>,
}

impl<T> Default for CallbackEmitter<T> {
    fn default() -> Self {
        Self {
            callbacks: Arc::new(Mutex::new(CallbackCollection::new())),
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
        self.callbacks.lock().len()
    }

    /// Returns `true` if there are no callbacks.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Downgrades the callback emitter to a [`WeakCallbackEmitter`].
    pub fn downgrade(&self) -> WeakCallbackEmitter<T> {
        WeakCallbackEmitter {
            callbacks: Arc::downgrade(&self.callbacks),
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
        self.callbacks.lock().insert(callback);
    }

    /// Unsubscribes a callback from the emitter.
    #[track_caller]
    pub fn unsubscribe(&self, ptr: CallbackPtr<T>) {
        self.callbacks.lock().remove(ptr);
    }

    /// Emits an event to all the callbacks.
    pub fn emit(&self, event: &T) {
        let callbacks = self.callbacks.lock();

        for callback in callbacks.iter() {
            if let Some(callback) = callback.upgrade() {
                callback.emit(event);
            }
        }
    }

    /// Clears all the callbacks, and calls them.
    ///
    /// This is used internally for emitting effect dependencies like signals, since effects
    /// always recreate dependencies when run.
    pub fn clear_and_emit(&self, event: &T) {
        let callbacks = mem::take(&mut *self.callbacks.lock());

        for callback in callbacks.into_iter() {
            if let Some(callback) = callback.upgrade() {
                callback.emit(event);
            }
        }
    }

    /// Clears all the callbacks.
    pub fn clear(&self) {
        self.callbacks.lock().callbacks.clear();
    }
}

impl CallbackEmitter {
    /// Tracks `self` in the current `effect`.
    pub fn track(&self) {
        self.downgrade().track();
    }
}

impl<T> Debug for CallbackEmitter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallbackEmitter")
            .field("callbacks", &self.callbacks)
            .finish()
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
        crate::effect::track_callback(self.clone());
    }
}

impl<T> Debug for WeakCallbackEmitter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakCallbackEmitter")
            .field("callbacks", &self.callbacks)
            .finish()
    }
}
