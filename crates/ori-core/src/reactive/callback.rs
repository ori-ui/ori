use std::{fmt::Debug, mem, panic::Location};

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
            if callback.callback.as_ptr() as CallbackPtr<T> == ptr {
                return true;
            }
        }

        false
    }

    fn insert(&mut self, callback: WeakCallback<T>) {
        if !self.contains(callback.callback.as_ptr() as CallbackPtr<T>) {
            self.callbacks.push(callback);
        }
    }

    fn remove(&mut self, ptr: CallbackPtr<T>) {
        let equals =
            |callback: &WeakCallback<T>| callback.callback.as_ptr() as CallbackPtr<T> != ptr;
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

#[cfg(feature = "multi-thread")]
type RawCallback<'a, T> = dyn FnMut(&T) + Send + 'a;
#[cfg(not(feature = "multi-thread"))]
type RawCallback<'a, T> = dyn FnMut(&T) + 'a;

type CallbackPtr<T> = *const T;
type Callbacks<T> = Lock<CallbackCollection<T>>;

/// A callback that can be called from any thread.
#[derive(Clone)]
pub struct Callback<'a, T = ()> {
    location: &'static Location<'static>,
    callback: Shared<Lock<RawCallback<'a, T>>>,
}

impl<'a, T> Callback<'a, T> {
    /// Creates a new callback.
    #[track_caller]
    pub fn new(callback: impl FnMut(&T) + Sendable + 'a) -> Self {
        Self {
            location: Location::caller(),
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
        WeakCallback {
            location: self.location,
            callback,
        }
    }

    pub fn as_ptr(&self) -> CallbackPtr<T> {
        Shared::as_ptr(&self.callback) as *const T
    }

    /// Calls the callback.
    pub fn emit(&self, event: &T) {
        self.callback.lock_mut()(event);
    }

    /// Returns the location where the callback was created.
    pub fn location(&self) -> &'static Location<'static> {
        self.location
    }
}

impl<'a, T> Default for Callback<'a, T> {
    fn default() -> Self {
        Callback::new(|_| {})
    }
}

impl<'a, T> Debug for Callback<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Callback")
            .field("location", &self.location)
            .finish()
    }
}

/// A weak reference to a [`Callback`].
///
/// This is usually created by [`Callback::downgrade`].
#[derive(Clone)]
pub struct WeakCallback<T = ()> {
    location: &'static Location<'static>,
    callback: Weak<Lock<RawCallback<'static, T>>>,
}

impl<T> WeakCallback<T> {
    /// Creates a new weak callback from a weak reference.
    #[track_caller]
    pub fn new(weak: Weak<Lock<RawCallback<'static, T>>>) -> Self {
        Self {
            location: Location::caller(),
            callback: weak,
        }
    }

    /// Tries to upgrade the weak callback to a [`Callback`].
    ///
    /// This will return `None` if all clones of the callback have been dropped.
    pub fn upgrade(&self) -> Option<Callback<T>> {
        Some(Callback {
            location: self.location,
            callback: self.callback.upgrade()?,
        })
    }

    /// Returns the raw pointer to the callback.
    pub fn as_ptr(&self) -> CallbackPtr<T> {
        Weak::as_ptr(&self.callback) as CallbackPtr<T>
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

    /// Returns the location where the callback was created.
    pub fn location(&self) -> &'static Location<'static> {
        self.location
    }
}

impl<T> Default for WeakCallback<T> {
    #[track_caller]
    fn default() -> Self {
        // FIXME: this is a hack to get a valid pointer
        // but it just doesn't feel right
        Callback::default().downgrade()
    }
}

impl<T> Debug for WeakCallback<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakCallback")
            .field("location", &self.location)
            .finish()
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
    #[track_caller]
    pub fn unsubscribe(&self, ptr: CallbackPtr<T>) {
        self.callbacks.lock_mut().remove(ptr);
    }

    /// Emits an event to all the callbacks.
    pub fn emit(&self, event: &T) {
        let callbacks = self.callbacks.lock_mut();

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
        let callbacks = mem::take(&mut *self.callbacks.lock_mut());

        for callback in callbacks.into_iter() {
            if let Some(callback) = callback.upgrade() {
                callback.emit(event);
            }
        }
    }

    /// Clears all the callbacks.
    pub fn clear(&self) {
        self.callbacks.lock_mut().callbacks.clear();
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
        super::effect::track_callback(self.clone());
    }
}

impl<T> Debug for WeakCallbackEmitter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakCallbackEmitter")
            .field("callbacks", &self.callbacks)
            .finish()
    }
}
