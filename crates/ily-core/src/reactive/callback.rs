use std::{
    collections::BTreeMap,
    mem,
    sync::{Arc, Mutex, Weak},
};

pub type RawCallback = dyn FnMut();
type CallbackPtr = *const Mutex<RawCallback>;
type Callbacks = Mutex<BTreeMap<CallbackPtr, WeakCallback>>;

#[derive(Clone)]
pub struct Callback {
    callback: Arc<Mutex<RawCallback>>,
}

impl Callback {
    pub fn new(callback: impl FnMut() + 'static) -> Self {
        Self {
            callback: Arc::new(Mutex::new(callback)),
        }
    }

    pub fn downgrade(&self) -> WeakCallback {
        WeakCallback {
            callback: Arc::downgrade(&self.callback),
        }
    }

    pub fn emit(&self) {
        if let Ok(mut callback) = self.callback.lock() {
            callback();
        }
    }
}

#[derive(Clone)]
pub struct WeakCallback {
    callback: Weak<Mutex<RawCallback>>,
}

impl WeakCallback {
    pub fn new(weak: Weak<Mutex<RawCallback>>) -> Self {
        Self { callback: weak }
    }

    pub fn upgrade(&self) -> Option<Callback> {
        Some(Callback {
            callback: self.callback.upgrade()?,
        })
    }

    pub fn as_ptr(&self) -> CallbackPtr {
        self.callback.as_ptr() as CallbackPtr
    }

    pub fn emit(&self) {
        if let Some(callback) = self.upgrade() {
            callback.emit();
        }
    }
}

#[derive(Clone, Default)]
pub struct WeakCallbackEmitter {
    callbacks: Weak<Callbacks>,
}

impl WeakCallbackEmitter {
    pub fn upgrade(&self) -> Option<CallbackEmitter> {
        Some(CallbackEmitter {
            callbacks: self.callbacks.upgrade()?,
        })
    }
}

#[derive(Default, Clone)]
pub struct CallbackEmitter {
    callbacks: Arc<Callbacks>,
}

impl CallbackEmitter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.callbacks.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn downgrade(&self) -> WeakCallbackEmitter {
        WeakCallbackEmitter {
            callbacks: Arc::downgrade(&self.callbacks),
        }
    }

    pub fn track(&self) {
        super::effect::track_callback(self.downgrade());
    }

    pub fn subscribe(&self, callback: &Callback) {
        self.subscribe_weak(callback.downgrade());
    }

    pub fn subscribe_weak(&self, callback: WeakCallback) {
        let ptr = callback.as_ptr();
        self.callbacks.lock().unwrap().insert(ptr, callback);
    }

    pub fn unsubscribe(&self, ptr: CallbackPtr) {
        self.callbacks.lock().unwrap().remove(&ptr);
    }

    pub fn emit(&self) {
        let callbacks = mem::take(&mut *self.callbacks.lock().unwrap());

        for callback in callbacks.into_values().rev() {
            if let Some(callback) = callback.upgrade() {
                callback.emit();
            }
        }
    }
}
