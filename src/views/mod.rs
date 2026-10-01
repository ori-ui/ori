//! Builtin [`View`](crate::View)s.

mod any;
mod build;
mod data;
mod effect;
mod either;
mod freeze;
mod keyed;
mod maybe;
mod memo;
mod mutate;
mod portal;
mod provide;
mod receive;
mod suspense;
mod task;

pub use any::any;
pub use build::{Build, build, context};
pub use data::{Map, With, map, map_with, with, with_default, without};
pub use effect::{Effects, WithEffect, effect, effects};
pub use either::Either;
pub use freeze::{Freeze, freeze};
pub use keyed::{Keyed, keyed};
pub use maybe::{Maybe, maybe};
pub use memo::{Memo, memo, memo_hashed};
pub use mutate::{Mutate, mutate};
pub use portal::{Portal, Teleport, portal, teleport};
pub use provide::{Provide, Using, provide, try_using, using, using_or_default};
pub use receive::{Receive, receive, receive_all};
pub use suspense::{Suspense, suspense};
pub use task::{Sink, Task, task};
