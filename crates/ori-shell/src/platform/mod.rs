//! Platform implementations for Ori Shell.

#[cfg(any(x11_platform, wayland_platform))]
mod linux;

#[cfg(x11_platform)]
pub mod x11;
