#![warn(missing_docs)]

//! A renderer using [`glow`].

mod mesh;
mod render;

#[cfg(all(feature = "glutin", not(target_arch = "wasm32")))]
mod glutin;

pub use render::*;

#[cfg(all(feature = "glutin", not(target_arch = "wasm32")))]
pub use glutin::{GlutinContext, GlutinError};

use std::fmt::Display;

/// An error that can occur when rendering.
#[derive(Debug)]
pub enum GlowError {
    /// No compatible config found.
    ConfigNotFound,
    /// Failed to request a device.
    Gl(String),
}

impl Display for GlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlowError::ConfigNotFound => write!(f, "No compatible config found"),
            GlowError::Gl(err) => write!(f, "{}", err),
        }
    }
}
