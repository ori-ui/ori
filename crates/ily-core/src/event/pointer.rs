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
    pub position: Vec2,
    pub pressed: bool,
    pub button: Option<PointerButton>,
    pub modifiers: Modifiers,
}

impl PointerEvent {
    pub fn is_pressed(&self, button: PointerButton) -> bool {
        self.pressed && self.button == Some(button)
    }

    pub fn is_released(&self, button: PointerButton) -> bool {
        !self.pressed && self.button == Some(button)
    }

    pub fn is_press(&self) -> bool {
        self.pressed && self.button.is_some()
    }

    pub fn is_release(&self) -> bool {
        !self.pressed && self.button.is_some()
    }
}
