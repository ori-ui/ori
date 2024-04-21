use std::hash::{Hash, Hasher};

use crate::layout::{Point, Vector};

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

/// A pointer was moved.
#[derive(Clone, Debug)]
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

/// A pointer button was pressed.
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
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
