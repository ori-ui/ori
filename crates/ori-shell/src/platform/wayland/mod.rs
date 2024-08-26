//! Wayland platform implementation.

mod error;
mod launch;

pub use error::WaylandError;
pub use launch::launch;

/// Check if the Wayland platform is available.
pub fn is_available() -> bool {
    wayland_client::Connection::connect_to_env().is_ok()
}
