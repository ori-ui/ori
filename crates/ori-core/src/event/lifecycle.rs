use std::cell::Cell;

/// Event emitted to a view when its hot state changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HotChanged(pub bool);

/// Event emitted when a view should be focused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RequestFocus {
    /// Focus the first view.
    First,
    /// Focus the last view.
    Last,
}

impl RequestFocus {
    /// Create a new focused event.
    pub fn new(first: bool) -> Self {
        if first {
            Self::First
        } else {
            Self::Last
        }
    }
}

/// Event emitted when focus should be switched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SwitchFocus {
    /// Switch focus to the next view.
    Next(Cell<bool>),
    /// Switch focus to the previous view.
    Prev(Cell<bool>),
}

impl SwitchFocus {
    /// Create a new switch focus event.
    pub fn new(next: bool) -> Self {
        if next {
            Self::Next(Cell::new(false))
        } else {
            Self::Prev(Cell::new(false))
        }
    }
}
