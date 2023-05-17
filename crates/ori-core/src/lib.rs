mod build;
mod children;
mod context;
mod event;
mod layout;
mod node;
mod reactive;
mod style;
mod unit;
mod view;
mod views;

pub use build::*;
pub use children::*;
pub use context::*;
pub use event::*;
pub use layout::*;
pub use node::*;
pub use reactive::*;
pub use style::*;
pub use unit::*;
pub use view::*;
pub use views::*;

pub use glam::*;

pub use tracing::{debug, error, info, trace, warn};

#[cfg(feature = "multi-thread")]
pub(crate) type Shared<T> = std::sync::Arc<T>;
#[cfg(feature = "multi-thread")]
pub(crate) type Weak<T> = std::sync::Weak<T>;
#[cfg(feature = "multi-thread")]
pub(crate) type Lock<T> = std::sync::Mutex<T>;
#[cfg(feature = "multi-thread")]
pub(crate) type Guard<'a, T> = std::sync::MutexGuard<'a, T>;

#[cfg(not(feature = "multi-thread"))]
pub(crate) type Shared<T> = std::rc::Rc<T>;
#[cfg(not(feature = "multi-thread"))]
pub(crate) type Weak<T> = std::rc::Weak<T>;
#[cfg(not(feature = "multi-thread"))]
pub(crate) type Lock<T> = std::cell::RefCell<T>;
#[cfg(not(feature = "multi-thread"))]
pub(crate) type Guard<'a, T> = std::cell::RefMut<'a, T>;

/// A trait that is implemented for all types that implement `Send`.
///
/// Send is only required when the `multi-thread` feature is enabled.
#[cfg(feature = "multi-thread")]
pub trait Sendable: Send {}
#[cfg(feature = "multi-thread")]
impl<T: Send> Sendable for T {}

/// A trait that is implemented for all types that implement `Send`.
///
/// Send is only required when the `multi-thread` feature is enabled.
#[cfg(not(feature = "multi-thread"))]
pub trait Sendable {}
#[cfg(not(feature = "multi-thread"))]
impl<T> Sendable for T {}

/// A trait that is implemented for all types that implement `Send + Sync`.
///
/// Send + Sync is only required when the `multi-thread` feature is enabled.
#[cfg(feature = "multi-thread")]
pub trait SendSync: Send + Sync {}
#[cfg(feature = "multi-thread")]
impl<T: Send + Sync> SendSync for T {}

/// A trait that is implemented for all types that implement `Send + Sync`.
///
/// Send + Sync is only required when the `multi-thread` feature is enabled.
#[cfg(not(feature = "multi-thread"))]
pub trait SendSync {}
#[cfg(not(feature = "multi-thread"))]
impl<T> SendSync for T {}

pub(crate) trait Lockable {
    type Item: ?Sized;

    fn lock_mut(&self) -> Guard<'_, Self::Item>;
}

impl<T: ?Sized> Lockable for Lock<T> {
    type Item = T;

    #[cfg(feature = "multi-thread")]
    #[track_caller]
    fn lock_mut(&self) -> Guard<'_, Self::Item> {
        self.lock().unwrap()
    }

    #[cfg(not(feature = "multi-thread"))]
    #[track_caller]
    fn lock_mut(&self) -> Guard<'_, Self::Item> {
        self.borrow_mut()
    }
}
