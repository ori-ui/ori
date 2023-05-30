use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    panic::Location,
};

use crate::{Callback, CallbackEmitter, Resource};

use super::effect;

/// A signal that can be read from, and subscribed to, see [`Signal`].
pub struct ReadSignal<T: 'static> {
    pub(crate) resource: Resource<T>,
    pub(crate) emitter: Resource<CallbackEmitter>,
}

impl<T: Send + Sync> ReadSignal<T> {
    /// Creates a new signal that must be manually disposed, see [`ReadSignal::dispose`].
    pub fn new_leaking(value: T) -> Self {
        Self {
            resource: Resource::new_leaking(value),
            emitter: Resource::new_leaking(CallbackEmitter::new()),
        }
    }

    /// Adds a reference to the signal.
    pub fn reference(self) {
        self.resource.reference();
        self.emitter.reference();
    }

    /// Tries to get a clone of the signal's value without tracking it, see [`ReadSignal::track`].
    pub fn try_get_untracked(self) -> Option<T>
    where
        T: Clone,
    {
        self.resource.get()
    }

    /// Tries to get a clone of the signal's value.
    pub fn try_get(self) -> Option<T>
    where
        T: Clone,
    {
        let value = self.try_get_untracked()?;
        self.track();
        Some(value)
    }

    /// Gets a clone of the signal's value without tracking it, see [`ReadSignal::track`].
    ///
    /// # Panics
    /// - If the signal has been disposed.
    #[track_caller]
    pub fn get_untracked(self) -> T
    where
        T: Clone,
    {
        match self.try_get_untracked() {
            Some(value) => value,
            None => panic!(
                "Signal::get() called on a dropped signal {:?}",
                self.resource.id()
            ),
        }
    }

    /// Gets a clone of the signal's value.
    ///
    /// # Panics
    /// - If the signal has been disposed.
    #[track_caller]
    pub fn get(self) -> T
    where
        T: Clone,
    {
        match self.try_get() {
            Some(value) => value,
            None => panic!(
                "Signal::get() called on a dropped signal {:?}",
                self.resource.id()
            ),
        }
    }

    /// Tracks the signal as a dependency of the current effect, see [`effect::track_callback`].
    pub fn track(self) {
        if let Some(emitter) = self.emitter.get() {
            effect::track_callback(emitter.downgrade());
        }
    }

    /// Tries to get the signal's emitter.
    pub fn emitter(self) -> Option<CallbackEmitter> {
        self.emitter.get()
    }

    /// Subscribes a [`Callback`] to the signal's emitter.
    pub fn subscribe(self, callback: &Callback) {
        if let Some(emitter) = self.emitter.get() {
            emitter.subscribe(callback);
        }
    }

    /// Disposes the signal.
    pub fn dispose(self) {
        self.resource.dispose();
        self.emitter.dispose();
    }
}

impl<T> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        Self {
            resource: self.resource.clone(),
            emitter: self.emitter.clone(),
        }
    }
}

impl<T> Copy for ReadSignal<T> {}

/// A signal that can be read from, subscribed to, and set, see [`ReadSignal`].
///
/// Signals implement [`Clone`] and [`Copy`].
pub struct Signal<T: 'static> {
    signal: ReadSignal<T>,
}

impl<T> Deref for Signal<T> {
    type Target = ReadSignal<T>;

    fn deref(&self) -> &Self::Target {
        &self.signal
    }
}

impl<T: Send + Sync> Signal<T> {
    /// Creates a new signal that must be manually disposed, see [`ReadSignal::dispose`].
    pub fn new_leaking(value: T) -> Self {
        Self {
            signal: ReadSignal::new_leaking(value),
        }
    }

    /// Creates a new signal from a [`ReadSignal`].
    ///
    /// **Note** that when read signals usually are read-only for a reason, and using this function
    /// is generally discouraged.
    pub fn from_read_signal(signal: ReadSignal<T>) -> Self {
        Self { signal }
    }

    /// Tries to set the signal's value without emitting, see [`Signal::emit`].
    #[track_caller]
    pub fn try_set_silent(self, value: T) -> Result<(), T> {
        match self.signal.resource.set(value) {
            Ok(_) => Ok(()),
            Err(value) => Err(value),
        }
    }

    /// Sets the signal's value without emitting, see [`Signal::emit`].
    ///
    /// # Panics
    /// - If the signal has been disposed.
    #[track_caller]
    pub fn set_silent(self, value: T) {
        if self.try_set_silent(value).is_err() {
            panic!("Signal::set_silent() called on a disposed signal");
        }
    }

    /// Tries to set the signal's value, see [`Signal::emit`].
    #[track_caller]
    pub fn try_set(self, value: T) -> Result<(), T> {
        self.try_set_silent(value)?;
        self.emit();
        Ok(())
    }

    /// Sets the signal's value, see [`Signal::emit`].
    ///
    /// # Panics
    /// - If the signal has been disposed.
    #[track_caller]
    pub fn set(self, value: T) {
        if self.try_set(value).is_err() {
            panic!("Signal::set() called on a disposed signal");
        }
    }

    /// Modifies the signal's value, see [`Modify`].
    pub fn modify(self) -> Modify<T>
    where
        T: Clone,
    {
        Modify::new(self)
    }

    /// Runs all callbacks subscribed to the signal's emitter.
    ///
    /// **Note** this will call [`CallbackEmitter::clear_and_emit`] on the emitter, which will
    /// clear the emitter's callbacks, and then run them. Dependencies will be re-tracked if
    /// accessed during the callbacks.
    #[track_caller]
    pub fn emit(self) {
        if let Some(emitter) = self.signal.emitter.get() {
            tracing::trace!("emitting signal at {}", Location::caller());
            emitter.clear_and_emit(&());
        }
    }
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal,
        }
    }
}

impl<T> Copy for Signal<T> {}

/// A guard that allows modifying a signal's value, see [`Signal::modify`].
///
/// This will get the signal's value when created, and set it when dropped, see [`ReadSignal::get`]
/// and [`Signal::set`].
pub struct Modify<T: Send + Sync + 'static> {
    signal: Signal<T>,
    value: Option<T>,
}

impl<T: Send + Sync + Clone> Modify<T> {
    /// Creates a new modify guard, see [`Signal::modify`].
    pub fn new(signal: Signal<T>) -> Self {
        Self {
            signal,
            value: Some(signal.get()),
        }
    }
}

impl<T: Send + Sync> Deref for Modify<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.signal.track();
        self.value.as_ref().unwrap()
    }
}

impl<T: Send + Sync> DerefMut for Modify<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.signal.track();
        self.value.as_mut().unwrap()
    }
}

impl<T: Send + Sync> Drop for Modify<T> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            self.signal.set(value);
        }
    }
}

/// A signal that is disposed when dropped, see [`ReadSignal`] and [`Signal`].
///
/// Signals implement [`Clone`] but not [`Copy`]. **Note** that cloning an [`OwnedSignal`] will increment its
/// reference count, and won't copy the signal's value.
pub struct OwnedSignal<T: 'static> {
    signal: Signal<T>,
}

impl<T> Deref for OwnedSignal<T> {
    type Target = Signal<T>;

    fn deref(&self) -> &Self::Target {
        &self.signal
    }
}

impl<T: Send + Sync> Clone for OwnedSignal<T> {
    fn clone(&self) -> Self {
        self.reference();

        Self {
            signal: self.signal,
        }
    }
}

impl<T: Send + Sync + Default> Default for OwnedSignal<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Send + Sync> OwnedSignal<T> {
    /// Creates a new owned signal.
    pub fn new(value: T) -> Self {
        Self {
            signal: Signal::new_leaking(value),
        }
    }

    /// Binds self to the given `signal`, this will discard the old signal, and set the internal
    /// signal to the given one.
    pub fn bind(&mut self, signal: Signal<T>) {
        // dispose the old signal
        self.signal.dispose();

        // bind to the new signal and increment the reference count
        self.signal = signal;
        self.reference();
    }
}

impl<T: Send + Sync> From<T> for OwnedSignal<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Drop for OwnedSignal<T> {
    #[track_caller]
    fn drop(&mut self) {
        self.resource.dispose();
        self.emitter.dispose();
    }
}

macro_rules! impl_signal {
    ($($type:ty),*) => {
        $(
            impl<T: Send + Sync + Clone + Debug> Debug for $type {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_struct(stringify!($type))
                        .field("resource", &self.resource)
                        .field("emitter", &self.emitter)
                        .finish()
                }
            }

            impl<T: Send + Sync + Clone + PartialEq> PartialEq for $type {
                fn eq(&self, other: &Self) -> bool {
                    self.resource == other.resource
                }
            }

            impl<T: Send + Sync + Clone + Eq> Eq for $type {}
        )*
    };
}

impl_signal!(ReadSignal<T>, Signal<T>, OwnedSignal<T>);
