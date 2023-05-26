use std::ops::Deref;

use once_cell::sync::OnceCell;

use crate::{OwnedSignal, Signal};

pub struct Atom<T: 'static> {
    signal: OnceCell<OwnedSignal<T>>,
    init: fn() -> T,
}

impl<T> Atom<T> {
    pub const fn new(init: fn() -> T) -> Self {
        Self {
            signal: OnceCell::new(),
            init,
        }
    }
}

impl<T: Send + Sync> Atom<T> {
    pub fn signal(&self) -> Signal<T> {
        *self.signal_ref()
    }

    pub fn signal_ref(&self) -> &Signal<T> {
        self.signal.get_or_init(|| OwnedSignal::new((self.init)()))
    }
}

impl<T: Send + Sync> Deref for Atom<T> {
    type Target = Signal<T>;

    fn deref(&self) -> &Self::Target {
        self.signal_ref()
    }
}

#[macro_export]
macro_rules! atom {
    ($init:expr) => {
        $crate::Atom::new(|| $init)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    static COUNTER: Atom<i32> = atom!(0);

    #[test]
    fn test_atom() {
        assert_eq!(COUNTER.get(), 0);
        *COUNTER.modify() += 1;
        assert_eq!(COUNTER.get(), 1);
    }
}
