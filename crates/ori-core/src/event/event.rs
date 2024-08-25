use std::any::Any;

use crate::command::Command;

use super::{
    CloseRequested, IsKey, KeyPressed, KeyReleased, PointerLeft, PointerMoved, PointerPressed,
    PointerReleased, PointerScrolled, WindowResized, WindowScaled,
};

/// An event that can be sent to a view.
#[derive(Debug)]
#[non_exhaustive]
pub enum Event {
    /// The window was resized.
    WindowResized(WindowResized),

    /// The window requested to be close.
    CloseRequested(CloseRequested),

    /// The window was scaled.
    WindowScaled(WindowScaled),

    /// A pointer moved.
    PointerMoved(PointerMoved),

    /// A pointer left the window.
    PointerLeft(PointerLeft),

    /// A pointer button was pressed.
    PointerPressed(PointerPressed),

    /// A pointer button was released.
    PointerReleased(PointerReleased),

    /// A pointer was scrolled.
    PointerScrolled(PointerScrolled),

    /// A keyboard key was pressed.
    KeyPressed(KeyPressed),

    /// A keyboard key was released.
    KeyReleased(KeyReleased),

    /// An animation frame has passed.
    Animate(f32),

    /// A command was sent.
    Command(Command),

    /// View state needs to be updated.
    Update,
}

impl Event {
    /// Check if the event is a command of a specific type.
    pub fn is_cmd<T: Any>(&self) -> bool {
        match self {
            Event::Command(cmd) => cmd.is::<T>(),
            _ => false,
        }
    }

    /// Try to get the command as a specific type.
    ///
    /// Returns `None` if the event is not a command or if the command is not of the specified type.
    pub fn cmd<T: Any>(&self) -> Option<&T> {
        match self {
            Event::Command(cmd) => cmd.get(),
            _ => None,
        }
    }

    /// Check if the event represents a key press of a specific key.
    pub fn is_key_pressed(&self, key: impl IsKey) -> bool {
        match self {
            Event::KeyPressed(pressed) => pressed.is_key(key),
            _ => false,
        }
    }

    /// Check if the event represents a key release of a specific key.
    pub fn is_key_released(&self, key: impl IsKey) -> bool {
        match self {
            Event::KeyReleased(released) => released.is_key(key),
            _ => false,
        }
    }
}
