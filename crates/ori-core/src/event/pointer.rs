use std::hash::{Hash, Hasher};

use crate::layout::{Point, Vector};

use super::Modifiers;

/// A unique pointer id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PointerId {
    id: u64,
}

impl PointerId {
    /// Create a new pointer id from a [`u64`].
    pub const fn from_u64(id: u64) -> Self {
        Self { id }
    }

    /// Create a new pointer id from a hashable value.
    pub fn from_hash(hash: &impl Hash) -> Self {
        let mut hasher = seahash::SeaHasher::new();
        hash.hash(&mut hasher);

        Self {
            id: hasher.finish(),
        }
    }

    /// Get the unique id as a [`u64`].
    pub const fn as_u64(&self) -> u64 {
        self.id
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

    /// The back button.
    Back,

    /// The forward button.
    Forward,

    /// Other buttons.
    Other(u16),
}

impl PointerButton {
    /// Create a new pointer button from a [`u16`].
    pub const fn from_u16(button: u16) -> Self {
        match button {
            1 => Self::Primary,
            2 => Self::Tertiary,
            3 => Self::Secondary,
            8 => Self::Back,
            9 => Self::Forward,
            button => Self::Other(button),
        }
    }
}

/// A pointer was moved.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct PointerMoved {
    /// The unique id of the pointer.
    pub id: PointerId,

    /// The position of the pointer.
    pub position: Point,

    /// The delta of the pointer.
    pub delta: Vector,

    /// The modifiers of the pointer.
    pub modifiers: Modifiers,
}

/// A pointer left the window.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct PointerLeft {
    /// The unique id of the pointer.
    pub id: PointerId,
}

/// A pointer button was pressed.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct PointerPressed {
    /// The unique id of the pointer.
    pub id: PointerId,

    /// The position of the pointer.
    pub position: Point,

    /// The button of the pointer.
    pub button: PointerButton,

    /// The modifiers of the pointer.
    pub modifiers: Modifiers,
}

/// A pointer button was released.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct PointerReleased {
    /// The unique id of the pointer.
    pub id: PointerId,

    /// The position of the pointer.
    pub position: Point,

    /// Whether the button was clicked.
    pub clicked: bool,

    /// The button of the pointer.
    pub button: PointerButton,

    /// The modifiers of the pointer.
    pub modifiers: Modifiers,
}

/// A pointer wheel was scrolled.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct PointerScrolled {
    /// The unique id of the pointer.
    pub id: PointerId,

    /// The position of the pointer.
    pub position: Point,

    /// The delta of the pointer.
    pub delta: Vector,

    /// The modifiers of the pointer.
    pub modifiers: Modifiers,
}
