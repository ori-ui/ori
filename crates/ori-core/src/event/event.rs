use std::any::Any;

use crate::{command::Command, view::ViewId, window::WindowId};

use super::{
    IsKey, KeyPressed, KeyReleased, PointerLeft, PointerMoved, PointerPressed, PointerReleased,
    PointerScrolled, WindowCloseRequested, WindowMaximized, WindowResized, WindowScaled,
};

/// A request to focus a view.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RequestFocus(pub WindowId, pub ViewId);

/// A target for focus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FocusTarget {
    /// Focus should be given to the next view in the focus chain.
    Next,

    /// Focus should be given to the previous view in the focus chain.
    Prev,

    /// Focus should be given to a specific view.
    View(ViewId),
}

/// An event that can be sent to a view.
#[derive(Debug)]
#[non_exhaustive]
pub enum Event {
    /// The window was resized.
    WindowResized(WindowResized),

    /// The window was scaled.
    WindowScaled(WindowScaled),

    /// The window was maximized.
    WindowMaximized(WindowMaximized),

    /// The window requested to be close.
    WindowCloseRequested(WindowCloseRequested),

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

    /// Focus should be switched to next view in the focus chain.
    FocusNext,

    /// Focus should be switched to previous view in the focus chain.
    FocusPrev,

    /// Focus is wanted by another view.
    ///
    /// A view receiving this event should give up focus.
    FocusWanted,

    /// Focus given to either a specific view or any focu
    FocusGiven(FocusTarget),

    /// An animation frame has passed.
    Animate(f32),

    /// A command was sent.
    Command(Command),

    /// Event sent when something has changed and the view should be given a chance to update.
    Notify,
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

    /// Check if the event wants to take focus.
    ///
    /// This is true for `FocusNext`, `FocusPrev`, and `FocusWanted`.
    #[rustfmt::skip]
    pub fn wants_focus(&self) -> bool {
        matches!(self, Event::FocusNext | Event::FocusPrev | Event::FocusWanted)
    }
}
