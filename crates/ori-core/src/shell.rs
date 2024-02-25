//! The shell for the [`Ui`], which is responsible for creating the [`CommandWaker`] and running the application.

use crate::{
    command::CommandWaker,
    ui::{Ui, UiBuilder},
    window::WindowDescriptor,
};

/// A collection of windows to be created.
///
/// This is built by the [`Launcher`](crate::launcher::Launcher), and passed to the [`Shell`] to create the windows.
pub struct Windows<T> {
    windows: Vec<(WindowDescriptor, UiBuilder<T>)>,
}

impl<T> Default for Windows<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Windows<T> {
    /// Create a new collection of windows.
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
        }
    }

    /// Add a window to the collection.
    pub fn push(&mut self, window: WindowDescriptor, ui: UiBuilder<T>) {
        self.windows.push((window, ui));
    }
}

impl<T> IntoIterator for Windows<T> {
    type Item = (WindowDescriptor, UiBuilder<T>);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.windows.into_iter()
    }
}

/// A shell for the UI.
pub trait Shell: Sized {
    /// The error type of the shell.
    type Error: std::error::Error;

    /// Initialize the shell.
    fn init() -> (Self, CommandWaker);

    /// Run the application.
    fn run<T>(self, data: T, ui: Ui<T>, windows: Windows<T>) -> Result<(), Self::Error>;
}
