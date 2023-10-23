//! A channel for sending commands to the event loop.

use std::{any::Any, fmt::Debug, sync::Arc};

use crossbeam_channel::{Receiver, Sender};

use crate::{log::warn_internal, view::ViewId};

/// A trait for waking the event loop.
///
/// This is used to wake the event loop when a command is sent through the [`CommandProxy`].
pub trait EventLoopWaker: Send + Sync {
    /// Wake the event loop.
    fn wake(&self);
}

impl Debug for dyn EventLoopWaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandWaker").finish()
    }
}

/// A command.
#[derive(Debug)]
pub struct Command {
    pub(crate) command: Box<dyn Any + Send>,
    pub(crate) target: Option<ViewId>,
    pub(crate) name: &'static str,
}

impl Command {
    /// Create a new command.
    pub fn new<T: Any + Send>(command: T) -> Self {
        Self {
            command: Box::new(command),
            target: None,
            name: std::any::type_name::<T>(),
        }
    }

    /// Create a new command with a target.
    pub fn new_targeted<T: Any + Send>(command: T, target: ViewId) -> Self {
        Self {
            command: Box::new(command),
            target: Some(target),
            name: std::any::type_name::<T>(),
        }
    }
}

/// A clonable channel for sending [`Command`]s.
#[derive(Clone, Debug)]
pub struct CommandProxy {
    tx: Sender<Command>,
    waker: Arc<dyn EventLoopWaker>,
}

impl CommandProxy {
    /// Create a new [`CommandProxy`] channel.
    pub fn new(waker: Arc<dyn EventLoopWaker>) -> (Self, Receiver<Command>) {
        let (tx, rx) = crossbeam_channel::unbounded();
        (Self { tx, waker }, rx)
    }

    /// Wake the event loop.
    pub fn wake(&self) {
        self.waker.wake();
    }

    /// Send a command without waking the event loop.
    ///
    /// This is almost never what you want to do. Use [`CommandProxy::cmd`] instead.
    pub fn cmd_silent(&self, command: Command) {
        if let Err(err) = self.tx.send(command) {
            warn_internal!("failed to send command: {}", err);
        }
    }

    /// Send a command.
    pub fn cmd(&self, command: impl Any + Send) {
        self.cmd_silent(Command::new(command));
        self.wake();
    }
}
