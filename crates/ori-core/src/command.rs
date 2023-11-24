//! A channel for sending commands to the event loop.

use std::{any::Any, fmt::Debug, sync::Arc};

use crossbeam_channel::{Receiver, Sender};

use crate::log::warn_internal;

/// A command.
#[derive(Debug)]
pub struct Command {
    pub(crate) command: Box<dyn Any + Send>,
    pub(crate) name: &'static str,
}

impl Command {
    /// Create a new command.
    pub fn new<T: Any + Send>(command: T) -> Self {
        Self {
            command: Box::new(command),

            name: std::any::type_name::<T>(),
        }
    }
}

/// A clonable channel for sending [`Command`]s.
#[derive(Clone)]
pub struct CommandProxy {
    tx: Sender<Command>,
    waker: Arc<dyn Fn()>,
}

impl CommandProxy {
    /// Create a new [`CommandProxy`] channel.
    pub fn new(waker: Arc<dyn Fn()>) -> (Self, Receiver<Command>) {
        let (tx, rx) = crossbeam_channel::unbounded();
        (Self { tx, waker }, rx)
    }

    /// Wake the event loop.
    pub fn wake(&self) {
        (self.waker)();
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

impl Debug for CommandProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandProxy").finish()
    }
}
