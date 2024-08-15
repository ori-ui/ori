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

use ori_core::view::{AnyView, BoxedView};

/// A builder for a user interface.
pub type UiBuilder<T> = Box<dyn FnMut(&mut T) -> BoxedView<T>>;

/// Trait for converting a type into a [`UiBuilder`].
pub trait IntoUiBuilder<V, P> {
    /// The data type of the returned view.
    type Data;

    /// Convert a type into it's requisite [`UiBuilder`].
    fn into_ui_builder(self) -> UiBuilder<Self::Data>;
}

impl<T, V, F> IntoUiBuilder<V, &mut T> for F
where
    F: FnMut(&mut T) -> V + 'static,
    V: AnyView<T> + 'static,
{
    type Data = T;

    fn into_ui_builder(mut self) -> UiBuilder<Self::Data> {
        Box::new(move |data| Box::new(self(data)))
    }
}

impl<V, F> IntoUiBuilder<V, ()> for F
where
    F: FnMut() -> V + 'static,
    V: AnyView<()> + 'static,
{
    type Data = ();

    fn into_ui_builder(mut self) -> UiBuilder<Self::Data> {
        Box::new(move |_| Box::new(self()))
    }
}
