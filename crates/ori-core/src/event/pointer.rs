use std::hash::{Hash, Hasher};

use crate::{
    layout::{Point, Vector},
    view::ViewId,
};

use super::Modifiers;

/// An event that is emitted when the hovered view changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HoveredChanged;

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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pointer {
    /// The unique id of the pointer.
    pub(crate) id: PointerId,
    /// The position of the pointer.
    pub(crate) position: Point,
    /// The view that is currently hovered by the pointer.
    pub(crate) hovered: Option<ViewId>,
}

impl Pointer {
    /// Create a new pointer.
    pub fn new(id: PointerId, position: Point) -> Self {
        Self {
            id,
            position,
            hovered: None,
        }
    }

    /// Get the unique id of the pointer.
    pub fn id(&self) -> PointerId {
        self.id
    }

    /// Get the position of the pointer.
    pub fn position(&self) -> Point {
        self.position
    }

    /// Get the view that is currently hovered by the pointer.
    pub fn hovered(&self) -> Option<ViewId> {
        self.hovered
    }

    /// Set the view that is currently hovered by the pointer.
    pub fn set_hovered(&mut self, hovered: Option<ViewId>) {
        self.hovered = hovered;
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

/// A pointer left the window.
pub struct PointerLeft {
    /// The unique id of the pointer.
    pub id: PointerId,
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
    /// The button of the pointer.
    pub button: PointerButton,
    /// The modifiers of the pointer.
    pub modifiers: Modifiers,
}

/// A pointer was clicked.
#[derive(Clone, Debug)]
pub struct PointerClicked {
    /// The unique id of the pointer.
    pub id: PointerId,
    /// The position of the pointer.
    pub position: Point,
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
