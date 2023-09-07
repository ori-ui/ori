#[allow(unused_macros)]
macro_rules! warn_internal {
    ($($tt:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing::warn!($($tt)*);

        #[cfg(not(feature = "tracing"))]
        eprintln!("[WARN]  ori-winit: {}", format_args!($($tt)*));
    };
}

#[allow(unused_macros)]
macro_rules! error_internal {
    ($($tt:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing::error!($($tt)*);

        #[cfg(not(feature = "tracing"))]
        eprintln!("[ERROR] ori-winit: {}", format_args!($($tt)*));
    };
}

#[allow(unused_imports)]
pub(crate) use {error_internal, warn_internal};
