mod emitter;

pub use emitter::*;

use std::{
    fmt::Debug,
    mem,
    panic::Location,
    sync::{Arc, Weak},
};

use parking_lot::Mutex;

type RawCallback<'a, T> = dyn FnMut(&T) + Send + 'a;
type CallbackPtr<T> = *const T;

/// A callback that can be called from any thread.
#[derive(Clone)]
pub struct Callback<'a, T = ()> {
    location: &'static Location<'static>,
    callback: Arc<Mutex<RawCallback<'a, T>>>,
}

impl<'a, T> Callback<'a, T> {
    /// Creates a new callback.
    #[track_caller]
    pub fn new(callback: impl FnMut(&T) + Send + 'a) -> Self {
        Self {
            location: Location::caller(),
            callback: Arc::new(Mutex::new(callback)),
        }
    }

    /// Downgrades the callback to a [`WeakCallback`].
    ///
    /// When the last strong reference is dropped, the callback will be dropped
    /// and all weak callbacks will be invalidated.
    pub fn downgrade(&self) -> WeakCallback<T> {
        type Lifetime<'a, T> = Weak<Mutex<RawCallback<'a, T>>>;
        type Static<T> = Weak<Mutex<RawCallback<'static, T>>>;

        let callback = unsafe {
            let weak = Arc::downgrade(&self.callback);

            // SAFETY: When the last strong reference is dropped, the callback will
            // be dropped and all weak callbacks will be invalidated. And can therefore
            // never be called. And since all strong references are tied to the lifetime
            // of the callback, it is safe to transmute the lifetime to static.
            mem::transmute::<Lifetime<'a, T>, Static<T>>(weak)
        };

        WeakCallback {
            location: self.location,
            callback,
        }
    }

    /// Returns the raw pointer to the callback.
    pub fn as_ptr(&self) -> CallbackPtr<T> {
        Arc::as_ptr(&self.callback) as *const T
    }

    /// Calls the callback.
    pub fn emit(&self, event: &T) {
        self.callback.lock()(event);
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
    callback: Weak<Mutex<RawCallback<'static, T>>>,
}

impl<T> WeakCallback<T> {
    /// Creates a new weak callback from a weak reference.
    #[track_caller]
    pub fn new(weak: Weak<Mutex<RawCallback<'static, T>>>) -> Self {
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
