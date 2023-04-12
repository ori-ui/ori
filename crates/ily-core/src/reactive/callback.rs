use std::{
    collections::BTreeMap,
    mem,
    sync::{Arc, Mutex, Weak},
};

pub type RawCallback<T> = dyn FnMut(&T);
type CallbackPtr<T> = *const Mutex<RawCallback<T>>;
type Callbacks<T> = Mutex<BTreeMap<CallbackPtr<T>, WeakCallback<T>>>;

#[derive(Clone)]
pub struct Callback<T = ()> {
    callback: Arc<Mutex<RawCallback<T>>>,
}

impl<T> Callback<T> {
    pub fn new(callback: impl FnMut(&T) + 'static) -> Self {
        Self {
            callback: Arc::new(Mutex::new(callback)),
        }
    }

    pub fn downgrade(&self) -> WeakCallback<T> {
        WeakCallback {
            callback: Arc::downgrade(&self.callback),
        }
    }

    pub fn emit(&self, event: &T) {
        if let Ok(mut callback) = self.callback.lock() {
            callback(event);
        }
    }
}

#[derive(Clone)]
pub struct WeakCallback<T = ()> {
    callback: Weak<Mutex<RawCallback<T>>>,
}

impl<T> WeakCallback<T> {
    pub fn new(weak: Weak<Mutex<RawCallback<T>>>) -> Self {
        Self { callback: weak }
    }

    pub fn upgrade(&self) -> Option<Callback<T>> {
        Some(Callback {
            callback: self.callback.upgrade()?,
        })
    }

    pub fn as_ptr(&self) -> CallbackPtr<T> {
        self.callback.as_ptr() as CallbackPtr<T>
    }

    pub fn emit(&self, event: &T) {
        if let Some(callback) = self.upgrade() {
            callback.emit(event);
        }
    }
}

pub struct WeakCallbackEmitter<T = ()> {
    callbacks: Weak<Callbacks<T>>,
}

impl<T> WeakCallbackEmitter<T> {
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
    pub fn track(&self) {
        super::effect::track_callback(self.clone());
    }
}

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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.callbacks.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn downgrade(&self) -> WeakCallbackEmitter<T> {
        WeakCallbackEmitter {
            callbacks: Arc::downgrade(&self.callbacks),
        }
    }

    pub fn subscribe(&self, callback: &Callback<T>) {
        self.subscribe_weak(callback.downgrade());
    }

    pub fn subscribe_weak(&self, callback: WeakCallback<T>) {
        let ptr = callback.as_ptr();
        self.callbacks.lock().unwrap().insert(ptr, callback);
    }

    pub fn unsubscribe(&self, ptr: CallbackPtr<T>) {
        self.callbacks.lock().unwrap().remove(&ptr);
    }

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
    pub fn track(&self) {
        super::effect::track_callback(self.downgrade());
    }
}
