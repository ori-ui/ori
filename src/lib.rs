#![warn(missing_docs, clippy::unwrap_used)]

//! Ori provides primitives for building Ori UI model.

mod action;
mod any;
mod build;
mod effect;
mod element;
mod future;
mod message;
mod provider;
mod proxy;
mod seq;
mod tree;
mod r#type;
mod view;

pub mod views;

pub use action::{Action, ActionCallback, ActionFuture};
pub use any::AnyView;
pub use build::{BuildMarker, BuildView};
pub use effect::{Effect, EffectSeq};
pub use element::{Base, Element, Is, Mut};
pub use message::{Message, ViewId};
pub use provider::Provider;
pub use proxy::{Proxied, Proxy};
pub use seq::{Elements, Keyed, ViewSeq, keyed};
pub use tree::{NodeId, Tracker, Tree};
pub use r#type::{get_relaxed_type_check, set_relaxed_type_check};
pub use view::{View, ViewMarker};
