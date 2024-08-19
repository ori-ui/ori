use crate::platform::linux::EglError;

/// Errors that can occur when interacting with X11.
#[derive(Debug)]
pub enum X11Error {
    /// An error occurred with the X11 connection.
    Connect(x11rb::errors::ConnectError),

    /// An error occurred with the X11 connection.
    Connection(x11rb::errors::ConnectionError),

    /// An error occurred with the X11 server.
    IdsExhausted,

    /// An error occurred with the X11 server.
    X11Error(x11rb::x11_utils::X11Error),

    /// An error occurred with the X11 server.
    Reply(x11rb::errors::ReplyError),

    /// An error occurred with egl.
    Egl(EglError),
}

impl From<x11rb::errors::ConnectError> for X11Error {
    fn from(err: x11rb::errors::ConnectError) -> Self {
        Self::Connect(err)
    }
}

impl From<x11rb::errors::ConnectionError> for X11Error {
    fn from(err: x11rb::errors::ConnectionError) -> Self {
        Self::Connection(err)
    }
}

impl From<x11rb::errors::ReplyOrIdError> for X11Error {
    fn from(err: x11rb::errors::ReplyOrIdError) -> Self {
        match err {
            x11rb::errors::ReplyOrIdError::IdsExhausted => X11Error::IdsExhausted,
            x11rb::errors::ReplyOrIdError::ConnectionError(err) => X11Error::Connection(err),
            x11rb::errors::ReplyOrIdError::X11Error(err) => X11Error::X11Error(err),
        }
    }
}

impl From<x11rb::errors::ReplyError> for X11Error {
    fn from(err: x11rb::errors::ReplyError) -> Self {
        Self::Reply(err)
    }
}

impl From<EglError> for X11Error {
    fn from(err: EglError) -> Self {
        Self::Egl(err)
    }
}

impl std::fmt::Display for X11Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            X11Error::Connect(err) => write!(f, "X11 connect error: {}", err),
            X11Error::Connection(err) => write!(f, "X11 connection error: {}", err),
            X11Error::IdsExhausted => write!(f, "X11 IDs exhausted"),
            X11Error::X11Error(err) => write!(f, "X11 error: {:?}", err),
            X11Error::Reply(err) => write!(f, "X11 reply error: {}", err),
            X11Error::Egl(err) => write!(f, "EGL error: {}", err),
        }
    }
}

impl std::error::Error for X11Error {}
