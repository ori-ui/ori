use jni::errors::Error as JniError;

use crate::platform::egl::EglError;

/// An error occured with the Android platform.
#[derive(Debug)]
pub enum AndroidError {
    /// The Android platform has not been initialized.
    NotInitialized,

    /// An error occurred with the EGL.
    Egl(EglError),

    /// An error occurred with the JNI.
    Jni(JniError),
}

impl From<EglError> for AndroidError {
    fn from(err: EglError) -> Self {
        Self::Egl(err)
    }
}

impl From<JniError> for AndroidError {
    fn from(err: JniError) -> Self {
        Self::Jni(err)
    }
}

impl std::fmt::Display for AndroidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "Android platform not initialized"),
            Self::Egl(err) => write!(f, "Android EGL error: {}", err),
            Self::Jni(err) => write!(f, "Android JNI error: {}", err),
        }
    }
}
