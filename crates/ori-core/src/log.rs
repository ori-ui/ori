//! Logging macros.

#[cfg(feature = "tracing")]
pub use tracing::{debug, error, info, trace, warn};

macro_rules! warn_internal {
    ($($tt:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing::warn!($($tt)*);

        #[cfg(not(feature = "tracing"))]
        eprintln!("ori [WARN] {}", format_args!($($tt)*));
    };
}

macro_rules! error_internal {
    ($($tt:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing::error!($($tt)*);

        #[cfg(not(feature = "tracing"))]
        eprintln!("ori [ERROR] {}", format_args!($($tt)*));
    };
}

pub(crate) use {error_internal, warn_internal};
