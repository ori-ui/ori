#![allow(clippy::module_inception)]
#![warn(missing_docs)]

//! Core library for the Ori UI framework.

pub mod canvas;
pub mod delegate;
pub mod event;
pub mod image;
pub mod layout;
pub mod log;
pub mod proxy;
pub mod rebuild;
pub mod style;
pub mod text;
pub mod transition;
pub mod ui;
pub mod view;
pub mod window;

pub mod views;

pub mod math {
    //! Math types and functions, powered by [`glam`].

    pub use glam::*;
}
