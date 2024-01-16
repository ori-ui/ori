//! Logging macros.

#[cfg(feature = "tracing")]
pub use tracing::{debug, error, info, trace, warn};

macro_rules! warn_internal {
    ($($tt:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing::warn!($($tt)*);

        #[cfg(not(feature = "tracing"))]
        eprintln!(" [WARN] ori: {}", format_args!($($tt)*));
    };
}

#[allow(unused_macros)]
macro_rules! error_internal {
    ($($tt:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing::error!($($tt)*);

        #[cfg(not(feature = "tracing"))]
        eprintln!("[ERROR] ori: {}", format_args!($($tt)*));
    };
}

#[allow(unused_imports)]
pub(crate) use {error_internal, warn_internal};
