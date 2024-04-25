#![deny(missing_docs)]
#![allow(clippy::module_inception)]

//! An application interface for the Ori library.

mod app;
mod builder;
mod command;
mod delegate;
mod request;

pub use app::*;
pub use builder::*;
pub use command::*;
pub use delegate::*;
pub use request::*;

use ori_core::view::BoxedView;

/// A builder for a user interface.
pub type UiBuilder<T> = Box<dyn FnMut(&mut T) -> BoxedView<T>>;
