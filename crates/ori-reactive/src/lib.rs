//! The reactivity system used by Ori.
//!
//! This crate provides six main primitives:
//! - [`Atom`] and [`AtomRef`] - A value that can be read from and written to.
//! - [`ReadSignal`], [`Signal`] and [`OwnedSignal`] - Handles to values managed by the runtime.
//! - [`Scope`] - A reactive context that can be used to create [`Signal`]s, and [`effect`]s.
//! - [`effect`] - A reactive effect that can be used to perform side effects.
//! - [`Event`] and [`EventSink`] - An event that can be used to send messages between components.
//! - [`Callback`] and [`CallbackEmitter`] - Callbacks that can be subscribed to and emitted.

mod atom;
mod atom_ref;
mod callback;
mod context;
pub mod effect;
mod event;
mod resource;
mod runtime;
mod scope;
mod signal;

pub use atom::*;
pub use atom_ref::*;
pub use callback::*;
pub use event::*;
pub use resource::*;
pub use runtime::*;
pub use scope::*;
pub use signal::*;

pub mod prelude {
    //! A collection of commonly used types and traits.

    pub use crate::atom;

    pub use crate::atom::Atom;
    pub use crate::atom_ref::AtomRef;
    pub use crate::callback::{Callback, CallbackEmitter};
    pub use crate::effect;
    pub use crate::event::Event;
    pub use crate::scope::Scope;
    pub use crate::signal::{OwnedSignal, ReadSignal, Signal};
}
