//! The shell for the [`Ui`], which is responsible for creating the [`CommandWaker`] and running the application.

use crate::{
    command::CommandWaker,
    ui::{Ui, UiBuilder},
    window::WindowDescriptor,
};

/// A shell for the UI.
pub trait Shell: Sized {
    /// The error type of the shell.
    type Error: std::error::Error;

    /// Initialize the shell.
    fn init() -> (Self, CommandWaker);

    /// Run the application.
    fn run<T>(
        self,
        ui: Ui<T>,
        windows: Vec<(WindowDescriptor, UiBuilder<T>)>,
    ) -> Result<(), Self::Error>;
}
