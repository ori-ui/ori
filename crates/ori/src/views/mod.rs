//! Builtin [`View`](crate::View)s.

mod r#async;
mod builder;
mod effect;
mod freeze;
mod handler;
mod map;
mod memo;
mod suspense;

pub use r#async::{AsyncHandler, on_build, task, task_with_proxy};
pub use builder::{Builder, builder};
pub use effect::{Effects, WithEffect, effects, with_effect};
pub use freeze::{Freeze, freeze};
pub use handler::{Handler, handler, on_event};
pub use map::{Map, map};
pub use memo::{Memo, hash_memo, memo};
pub use suspense::{Suspense, suspense};
