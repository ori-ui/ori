mod mesh;
mod render;

use std::fmt::Display;

pub use render::*;

#[derive(Debug)]
pub enum GlowError {
    Glutin(glutin::error::Error),
    ConfigNotFound,
    Gl(String),
}

impl From<glutin::error::Error> for GlowError {
    fn from(err: glutin::error::Error) -> Self {
        Self::Glutin(err)
    }
}

impl Display for GlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlowError::Glutin(err) => write!(f, "{}", err),
            GlowError::ConfigNotFound => write!(f, "No compatible config found"),
            GlowError::Gl(err) => write!(f, "{}", err),
        }
    }
}
