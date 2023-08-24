use std::hash::{Hash, Hasher};

use glam::Vec2;

use super::Modifiers;

/// A unique pointer id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PointerId {
    id: u64,
}

impl PointerId {
    /// Create a new pointer id from a hashable value.
    pub fn from_hash(hash: &impl Hash) -> Self {
        let mut hasher = seahash::SeaHasher::new();
        hash.hash(&mut hasher);

        Self {
            id: hasher.finish(),
        }
    }
}

/// A pointer.
#[derive(Clone, Debug, PartialEq)]
pub struct Pointer {
    /// The unique id of the pointer.
    pub(crate) id: PointerId,
    /// The position of the pointer.
    pub(crate) position: Vec2,
}

impl Pointer {
    /// Create a new pointer.
    pub fn new(id: PointerId, position: Vec2) -> Self {
        Self { id, position }
    }

    /// Get the unique id of the pointer.
    pub fn id(&self) -> PointerId {
        self.id
    }

    /// Get the position of the pointer.
    pub fn position(&self) -> Vec2 {
        self.position
    }
}

/// A pointer button.
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

/// A pointer event.
#[derive(Clone, Debug)]
pub struct PointerEvent {
    /// The unique id of the pointer.
    pub id: PointerId,
    /// The position of the pointer.
    pub position: Vec2,
    /// The delta of the pointer.
    pub delta: Vec2,
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
    /// Create a new empty pointer event.
    pub fn new(id: PointerId) -> Self {
        Self {
            id,
            position: Vec2::ZERO,
            delta: Vec2::ZERO,
            scroll_delta: Vec2::ZERO,
            pressed: false,
            left: false,
            button: None,
            modifiers: Modifiers::default(),
        }
    }

    /// Returns true if the event is a move event.
    pub fn is_move(&self) -> bool {
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
