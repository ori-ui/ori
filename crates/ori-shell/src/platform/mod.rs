//! Platform implementations for Ori Shell.

#[cfg(any(x11_platform, wayland_platform, android_platform))]
mod egl;
#[cfg(any(x11_platform, wayland_platform))]
mod linux;

#[cfg(android_platform)]
pub mod android;
#[cfg(wayland_platform)]
pub mod wayland;
#[cfg(x11_platform)]
pub mod x11;
