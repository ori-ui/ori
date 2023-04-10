use std::{
    cmp::Ordering,
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use crate::CallbackEmitter;

pub struct ReadSignal<T: ?Sized> {
    value: Mutex<Arc<T>>,
    emitter: CallbackEmitter,
}

impl<T> ReadSignal<T> {
    pub fn new(value: T) -> Self {
        Self::new_arc(Arc::new(value))
    }
}

impl<T: ?Sized> ReadSignal<T> {
    pub fn new_arc(value: Arc<T>) -> Self {
        Self {
            value: Mutex::new(value),
            emitter: CallbackEmitter::new(),
        }
    }

    pub fn emitter(&self) -> &CallbackEmitter {
        &self.emitter
    }

    pub fn track(&self) {
        self.emitter.track();
    }

    pub fn get(&self) -> Arc<T> {
        self.emitter.track();
        self.get_untracked()
    }

    pub fn get_untracked(&self) -> Arc<T> {
        self.value.lock().unwrap().clone()
    }
}

impl<T: Clone> ReadSignal<T> {
    pub fn cloned(&self) -> T {
        self.get().as_ref().clone()
    }
}

pub struct Signal<T: ?Sized>(ReadSignal<T>);

impl<T> Signal<T> {
    pub fn new(value: T) -> Self {
        Self(ReadSignal::new(value))
    }

    pub fn set(&self, value: T) {
        self.set_arc(Arc::new(value));
    }

    pub fn set_silent(&self, value: T) {
        self.set_arc_silent(Arc::new(value));
    }
}

impl<T: ?Sized> Signal<T> {
    pub fn new_arc(value: Arc<T>) -> Self {
        Self(ReadSignal::new_arc(value))
    }

    pub fn set_arc(&self, value: Arc<T>) {
        self.set_arc_silent(value.clone());
        self.emit();
    }

    pub fn set_arc_silent(&self, value: Arc<T>) {
        *self.value.lock().unwrap() = value;
    }

    pub fn emit(&self) {
        self.emitter.emit();
    }
}

impl<T: ?Sized> Deref for Signal<T> {
    type Target = ReadSignal<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct Modify<'a, T> {
    value: Option<T>,
    signal: &'a Signal<T>,
}

impl<'a, T> Deref for Modify<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref().unwrap()
    }
}

impl<'a, T> DerefMut for Modify<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.as_mut().unwrap()
    }
}

/// When the [`Modify`] is dropped, update the [`Signal`].
impl<'a, T> Drop for Modify<'a, T> {
    fn drop(&mut self) {
        self.signal.set(self.value.take().unwrap());
    }
}

impl<T: Clone> Signal<T> {
    /// Returns a [`Modify`] that can be used to modify the value of the [`Signal`].
    /// When the [`Modify`] is dropped, the [`Signal`] will be updated.
    pub fn modify(&self) -> Modify<'_, T> {
        Modify {
            value: Some(self.get().as_ref().clone()),
            signal: self,
        }
    }
}

pub struct SharedSignal<T: ?Sized>(Arc<Signal<T>>);

impl<T> SharedSignal<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(Signal::new(value)))
    }
}

impl<T: ?Sized> SharedSignal<T> {
    pub fn new_arc(value: Arc<T>) -> Self {
        Self(Arc::new(Signal::new_arc(value)))
    }
}

impl<T: ?Sized> Clone for SharedSignal<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: ?Sized> Deref for SharedSignal<T> {
    type Target = Signal<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Debug + ?Sized> Debug for ReadSignal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ReadSignal").field(&self.get()).finish()
    }
}

impl<T: Debug + ?Sized> Debug for Signal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Signal").field(&self.get()).finish()
    }
}

impl<T: Debug + ?Sized> Debug for SharedSignal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SharedSignal").field(&self.get()).finish()
    }
}

impl<T: Default> Default for ReadSignal<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Default> Default for Signal<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Default> Default for SharedSignal<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: PartialEq + ?Sized> PartialEq for ReadSignal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T: PartialEq + ?Sized> PartialEq for Signal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T: PartialEq + ?Sized> PartialEq for SharedSignal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T: PartialEq + ?Sized> PartialEq<T> for ReadSignal<T> {
    fn eq(&self, other: &T) -> bool {
        self.get().as_ref() == other
    }
}

impl<T: PartialEq + ?Sized> PartialEq<T> for Signal<T> {
    fn eq(&self, other: &T) -> bool {
        self.get().as_ref() == other
    }
}

impl<T: PartialEq + ?Sized> PartialEq<T> for SharedSignal<T> {
    fn eq(&self, other: &T) -> bool {
        self.get().as_ref() == other
    }
}

impl<T: Eq + ?Sized> Eq for ReadSignal<T> {}
impl<T: Eq + ?Sized> Eq for Signal<T> {}
impl<T: Eq + ?Sized> Eq for SharedSignal<T> {}

impl<T: PartialOrd + ?Sized> PartialOrd for ReadSignal<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get().partial_cmp(&other.get())
    }
}

impl<T: PartialOrd + ?Sized> PartialOrd for Signal<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get().partial_cmp(&other.get())
    }
}

impl<T: PartialOrd + ?Sized> PartialOrd for SharedSignal<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get().partial_cmp(&other.get())
    }
}

impl<T: PartialOrd + ?Sized> PartialOrd<T> for ReadSignal<T> {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        self.get().as_ref().partial_cmp(other)
    }
}

impl<T: PartialOrd + ?Sized> PartialOrd<T> for Signal<T> {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        self.get().as_ref().partial_cmp(other)
    }
}

impl<T: PartialOrd + ?Sized> PartialOrd<T> for SharedSignal<T> {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        self.get().as_ref().partial_cmp(other)
    }
}

impl<T: Ord + ?Sized> Ord for ReadSignal<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get().cmp(&other.get())
    }
}

impl<T: Ord + ?Sized> Ord for Signal<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get().cmp(&other.get())
    }
}

impl<T: Ord + ?Sized> Ord for SharedSignal<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get().cmp(&other.get())
    }
}

impl<T: Hash + ?Sized> Hash for ReadSignal<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get().hash(state);
    }
}

impl<T: Hash + ?Sized> Hash for Signal<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get().hash(state);
    }
}

impl<T: Hash + ?Sized> Hash for SharedSignal<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get().hash(state);
    }
}
