use glam::Vec2;

use crate::Modifiers;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointerButton {
    Primary,
    Secondary,
    Tertiary,
    Other(u16),
}

#[derive(Clone, Debug, Default)]
pub struct PointerEvent {
    /// The unique id of the pointer.
    pub id: u64,
    /// The position of the pointer.
    pub position: Vec2,
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
