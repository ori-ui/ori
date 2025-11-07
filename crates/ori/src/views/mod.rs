//! Builtin [`View`](crate::View)s.

mod actor;
mod focus;
mod freeze;
mod handler;
mod memo;
mod with;

pub use actor::{Actor, actor, task};
pub use focus::{Focus, focus};
pub use freeze::{Freeze, freeze};
pub use handler::{EventHandler, Handler, handler, on_event};
pub use memo::{Memo, hash_memo, memo};
pub use with::{With, with};
