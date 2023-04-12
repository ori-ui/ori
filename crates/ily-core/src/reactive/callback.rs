use std::{
    collections::BTreeMap,
    mem,
    sync::{Arc, Mutex, Weak},
};

type RawCallback<T> = dyn FnMut(&T) + Send + Sync;
type CallbackPtr<T> = *const Mutex<RawCallback<T>>;
type Callbacks<T> = Mutex<BTreeMap<CallbackPtr<T>, WeakCallback<T>>>;

/// A callback that can be called from any thread.
#[derive(Clone)]
pub struct Callback<T = ()> {
    callback: Arc<Mutex<RawCallback<T>>>,
}

impl<T> Callback<T> {
    /// Creates a new callback.
    pub fn new(callback: impl FnMut(&T) + Send + Sync + 'static) -> Self {
        Self {
            callback: Arc::new(Mutex::new(callback)),
        }
    }

    /// Downgrades the callback to a [`WeakCallback`].
    ///
    /// When the last strong reference is dropped, the callback will be dropped
    /// and all weak callbacks will be invalidated.
    pub fn downgrade(&self) -> WeakCallback<T> {
        WeakCallback {
            callback: Arc::downgrade(&self.callback),
        }
    }

    /// Calls the callback.
    pub fn emit(&self, event: &T) {
        if let Ok(mut callback) = self.callback.lock() {
            callback(event);
        }
    }
}

/// A weak reference to a [`Callback`].
///
/// This is usually created by [`Callback::downgrade`].
#[derive(Clone)]
pub struct WeakCallback<T = ()> {
    callback: Weak<Mutex<RawCallback<T>>>,
}

impl<T> WeakCallback<T> {
    /// Creates a new weak callback from a weak reference.
    pub fn new(weak: Weak<Mutex<RawCallback<T>>>) -> Self {
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
            callbacks: Arc::new(Mutex::new(BTreeMap::new())),
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
        self.callbacks.lock().unwrap().len()
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
    pub fn subscribe(&self, callback: &Callback<T>) {
        self.subscribe_weak(callback.downgrade());
    }

    /// Subscribes a weak callback to the emitter.
    pub fn subscribe_weak(&self, callback: WeakCallback<T>) {
        let ptr = callback.as_ptr();
        self.callbacks.lock().unwrap().insert(ptr, callback);
    }

    /// Unsubscribes a callback from the emitter.
    pub fn unsubscribe(&self, ptr: CallbackPtr<T>) {
        self.callbacks.lock().unwrap().remove(&ptr);
    }

    /// Clears all the callbacks, and calls them.
    pub fn emit(&self, event: &T) {
        let callbacks = mem::take(&mut *self.callbacks.lock().unwrap());

        for callback in callbacks.into_values().rev() {
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
