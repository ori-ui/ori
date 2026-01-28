//! Builtin [`View`](crate::View)s.

mod any;
mod builder;
mod effect;
mod freeze;
mod memo;
mod provide;
mod receive;
mod state;
mod suspense;
mod task;

pub use any::any;
pub use builder::{Builder, build, build_with_context};
pub use effect::{Effects, WithEffect, effects, with_effect};
pub use freeze::{Freeze, freeze};
pub use memo::{Memo, memo, memo_hashed};
pub use provide::{Provide, Using, provide, try_using, using, using_or_default};
pub use receive::{Receive, receive, receive_any};
pub use state::{Map, With, map, map_with, with, with_default};
pub use suspense::{Suspense, suspense};
pub use task::{Sink, Task, task};
