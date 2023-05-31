use crate::Modifiers;

/// A keyboard event.
#[derive(Clone, Debug, Default)]
pub struct KeyboardEvent {
    /// Whether the key was pressed or released.
    pub pressed: bool,
    /// The key that was pressed or released.
    pub key: Option<Key>,
    /// The text that was entered.
    pub text: Option<String>,
    /// The modifiers that were active.
    pub modifiers: Modifiers,
}

impl KeyboardEvent {
    /// Check if the `key` is pressed.
    pub fn is_pressed(&self, key: Key) -> bool {
        self.pressed && self.key == Some(key)
    }

    /// Check if the `key` is released.
    pub fn is_released(&self, key: Key) -> bool {
        !self.pressed && self.key == Some(key)
    }

    /// Check if the event is a key press.
    pub fn is_press(&self) -> bool {
        self.pressed && self.key.is_some()
    }

    /// Check if the event is a key release.
    pub fn is_release(&self) -> bool {
        !self.pressed && self.key.is_some()
    }
}

/// A keyboard key.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    // Alphabetical keys
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    // Number keys
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    // Symbol keys
    Plus,
    Minus,
    Comma,
    Period,
    // Arrow keys
    Left,
    Right,
    Up,
    Down,
    // Special keys
    Escape,
    Tab,
    Space,
    Backspace,
    Enter,
    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    CapsLock,
    // Modifier keys
    Shift,
    Ctrl,
    Alt,
    Meta,
}
