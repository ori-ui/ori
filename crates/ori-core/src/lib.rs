#![allow(clippy::module_inception)]
#![warn(missing_docs)]

//! Core library for the Ori UI framework.

pub mod canvas;
pub mod clipboard;
pub mod command;
pub mod debug;
pub mod delegate;
pub mod event;
pub mod image;
pub mod launcher;
pub mod layout;
pub mod rebuild;
pub mod shell;
pub mod text;
pub mod theme;
pub mod transition;
pub mod ui;
pub mod view;
pub mod window;

pub mod views;

#[allow(unused_imports)]
pub mod log;
