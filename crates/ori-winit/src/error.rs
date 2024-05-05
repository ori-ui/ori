use std::fmt::Display;

use softbuffer::SoftBufferError;
use winit::error::{EventLoopError, OsError};

/// An error that can occur when running an application.
#[derive(Debug)]
pub enum Error {
    /// An OS error.
    OsError(OsError),

    /// An error occurred with the event loop.
    EventLoop(EventLoopError),

    /// An error occured with softbuffer.
    SoftBuffer(SoftBufferError),
}

impl From<OsError> for Error {
    fn from(err: OsError) -> Self {
        Self::OsError(err)
    }
}

impl From<EventLoopError> for Error {
    fn from(err: EventLoopError) -> Self {
        Self::EventLoop(err)
    }
}

impl From<SoftBufferError> for Error {
    fn from(err: SoftBufferError) -> Self {
        Self::SoftBuffer(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::OsError(err) => write!(f, "{}", err),
            Error::EventLoop(err) => write!(f, "{}", err),
            Error::SoftBuffer(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {}
