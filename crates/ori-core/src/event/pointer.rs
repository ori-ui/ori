use glam::Vec2;

use crate::Modifiers;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointerButton {
    /// The primary button, usually the left mouse button or the touch screen.
    Primary,
    /// The secondary button, usually the right mouse button.
    Secondary,
    /// The tertiary button, usually the middle mouse button.
    Tertiary,
    /// Other buttons.
    Other(u16),
}

#[derive(Clone, Debug, Default)]
pub struct PointerEvent {
    /// The unique id of the pointer.
    pub id: u64,
    /// The position of the pointer.
    pub position: Vec2,
    /// The delta of the pointer wheel.
    pub scroll_delta: Vec2,
    /// Whether the pointer is pressed.
    pub pressed: bool,
    /// Whether the pointer left the window.
    pub left: bool,
    /// The button that was pressed or released.
    pub button: Option<PointerButton>,
    /// The modifiers that were active when the event was triggered.
    pub modifiers: Modifiers,
}

impl PointerEvent {
    /// Returns true if the event is a motion event.
    pub fn is_motion(&self) -> bool {
        self.scroll_delta == Vec2::ZERO && self.button.is_none()
    }

    /// Returns true if `button` was pressed.
    pub fn is_pressed(&self, button: PointerButton) -> bool {
        self.pressed && self.button == Some(button)
    }

    /// Returns true if `button` was released.
    pub fn is_released(&self, button: PointerButton) -> bool {
        !self.pressed && self.button == Some(button)
    }

    /// Returns true if any button was pressed.
    pub fn is_press(&self) -> bool {
        self.pressed && self.button.is_some()
    }

    /// Returns true if any button was released.
    pub fn is_release(&self) -> bool {
        !self.pressed && self.button.is_some()
    }
}
