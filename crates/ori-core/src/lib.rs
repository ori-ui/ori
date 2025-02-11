#![allow(clippy::module_inception)]
#![warn(missing_docs)]

//! Core library for the Ori UI framework.

pub mod canvas;
pub mod clipboard;
pub mod command;
pub mod context;
pub mod event;
pub mod image;
pub mod layout;
pub mod rebuild;
pub mod style;
pub mod text;
pub mod transition;
pub mod view;
pub mod window;

pub mod views;

pub use tracing as log;

// this is a workaround for proc-macros in doc tests
//  - https://github.com/bkchr/proc-macro-crate/issues/14
extern crate self as ori_core;
