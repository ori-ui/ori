use std::{
    collections::BTreeMap,
    mem,
    sync::{Arc, Mutex, Weak},
};

pub type RawCallback = dyn FnMut() + Send + Sync;
pub type Callback = Arc<Mutex<RawCallback>>;
pub type WeakCallback = Weak<Mutex<RawCallback>>;
type CallbackPtr = *const Mutex<RawCallback>;
type Callbacks = Mutex<BTreeMap<CallbackPtr, WeakCallback>>;

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
        crate::effect::track_callback(self.downgrade());
    }

    pub fn subscribe(&self, callback: &Callback) {
        self.subscribe_weak(Arc::downgrade(callback));
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
                callback.lock().unwrap()();
            }
        }
    }
}
