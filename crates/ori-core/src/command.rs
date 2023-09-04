//! A channel for sending commands to the event loop.

use std::{any::Any, fmt::Debug, sync::Arc};

use crossbeam_channel::{Receiver, Sender};

use crate::log::warn_internal;

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
#[derive(Clone, Debug)]
pub struct CommandProxy {
    pub(crate) tx: Sender<Command>,
    pub(crate) rx: Receiver<Command>,
    pub(crate) waker: Arc<dyn EventLoopWaker>,
}

impl CommandProxy {
    /// Create a new [`CommandProxy`] channel.
    pub fn new(waker: Arc<dyn EventLoopWaker>) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        Self { tx, rx, waker }
    }

    pub(crate) fn send_internal(&self, command: Command) {
        if let Err(err) = self.tx.send(command) {
            warn_internal!("failed to send command: {}", err);
        }
    }

    /// Send a command.
    pub fn cmd(&self, command: impl Any + Send) {
        self.send_internal(Command::new(command));

        self.waker.wake();
    }
}
