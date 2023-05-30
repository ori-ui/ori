use std::ops::Deref;

use once_cell::sync::OnceCell;

use crate::{OwnedSignal, Signal};

/// A thread-safe, reactive value.
///
/// This is similar to a [`Signal`](crate::Signal), but can be created at compile-time
/// with the [`atom!`](crate::atom!) macro. This allows you to create static values
/// that can be read from and written to.
///
/// Atoms implement [`Deref`] to [`Signal`], so you can use them in any place that
/// expects a [`Signal`].
///
/// # Example
/// ```
/// # use ori_reactive::prelude::*;
/// // this is a static Atom, created with the `atom!` macro
/// static COUNTER: Atom<i32> = atom!(0);
///
/// // we can read the value with `get`
/// assert_eq!(COUNTER.get(), 0);
///
/// // and write the value with `modify`
/// *COUNTER.modify() += 1;
///
/// // and read it again
/// assert_eq!(COUNTER.get(), 1);
/// ```
pub struct Atom<T: 'static> {
    signal: OnceCell<OwnedSignal<T>>,
    init: fn() -> T,
}

impl<T> Atom<T> {
    /// Creates a new [`Atom`] with the given initializer.
    ///
    /// See [`atom!`](crate::atom!) for more information.
    pub const fn new(init: fn() -> T) -> Self {
        Self {
            signal: OnceCell::new(),
            init,
        }
    }
}

impl<T: Send + Sync> Atom<T> {
    /// Gets the [`Signal`] for this [`Atom`].
    pub fn signal(&self) -> Signal<T> {
        *self.signal_ref()
    }

    /// Gets a reference to the [`Signal`] for this [`Atom`].
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

/// Creates a new [`Atom`] or [`AtomRef`] with the given initializer.
///
/// This macro is used to create static [`Atom`]s and [`AtomRef`]s. It takes a single
/// expression, which is the initial value of the [`Atom`] or [`AtomRef`].
///
/// If the expression is prefixed with `ref`, then an [`AtomRef`] will be created,
/// otherwise, an [`Atom`] will be created.
///
/// [`AtomRef`]: crate::AtomRef
#[macro_export]
macro_rules! atom {
    ($init:expr) => {
        $crate::Atom::new(|| $init)
    };
    (ref $init:expr) => {
        $crate::AtomRef::new(|| $init)
    };
}
