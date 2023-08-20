//! Logging macros.

#[cfg(feature = "tracing")]
pub use tracing::{debug, error, info, trace, warn};

macro_rules! warn_internal {
    ($($tt:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing::warn!($($tt)*);

        #[cfg(not(feature = "tracing"))]
        eprintln!("ori [WARNING] {}", format_args!($($tt)*));
    };
}

pub(crate) use warn_internal;
