use crate::{
    event::{PointerButton, PointerId},
    layout::Point,
    view::ViewId,
};

/// The state of a pointer.
#[derive(Clone, Debug, PartialEq)]
pub struct Pointer {
    id: PointerId,
    pressed: Vec<(PointerButton, Point)>,

    /// The position of the pointer.
    ///
    /// You probably don't want to set this directly.
    pub position: Point,

    /// The view the pointer is over.
    ///
    /// You probably don't want to set this directly.
    pub hovering: Option<ViewId>,
}

impl Pointer {
    /// Create a new pointer.
    pub fn new(id: PointerId, position: Point) -> Self {
        Self {
            id,
            pressed: Vec::new(),
            position,
            hovering: None,
        }
    }

    /// Get the unique identifier of the pointer.
    pub fn id(&self) -> PointerId {
        self.id
    }

    /// Get whether a button is pressed.
    pub fn is_pressed(&self, button: PointerButton) -> bool {
        self.pressed.iter().any(|(b, _)| b == &button)
    }

    /// Press a button.
    ///
    /// This is rarely what you want to do, do not use this unless you
    /// really know what you are doing.
    pub fn press(&mut self, button: PointerButton) {
        if !self.is_pressed(button) {
            self.pressed.push((button, self.position));
        }
    }

    /// Release a button, returning whether the button was clicked.
    ///
    /// This is rarely what you want to do, do not use this unless you
    /// really know what you are doing.
    pub fn release(&mut self, button: PointerButton) -> bool {
        match self.pressed.iter().position(|(b, _)| b == &button) {
            Some(index) => {
                let (_, position) = self.pressed.remove(index);
                let distance = position.distance(self.position);
                distance < 10.0
            }
            None => false,
        }
    }
}
