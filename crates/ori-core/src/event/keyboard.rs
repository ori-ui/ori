use super::Modifiers;

/// An event fired when a key is pressed.
#[derive(Clone, Debug, Default)]
pub struct KeyPressed {
    /// The key that was pressed or released.
    pub code: Option<Code>,
    /// The text that was entered.
    pub text: Option<String>,
    /// The modifiers that were active.
    pub modifiers: Modifiers,
}

impl KeyPressed {
    /// Check if the `key` is pressed.
    pub fn is(&self, key: Code) -> bool {
        self.code == Some(key)
    }
}

/// An event fired when a key is released.
#[derive(Clone, Debug, Default)]
pub struct KeyReleased {
    /// The key that was pressed or released.
    pub code: Option<Code>,
    /// The modifiers that were active.
    pub modifiers: Modifiers,
}

impl KeyReleased {
    /// Check if the `key` is released.
    pub fn is(&self, key: Code) -> bool {
        self.code == Some(key)
    }
}

/// A keyboard key.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Code {
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
    Asterisk,
    Slash,
    Backslash,
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

impl Code {
    /// Get the digit of the key, if it is a digit.
    pub const fn as_digit(self) -> Option<u8> {
        match self {
            Self::Key0 => Some(0),
            Self::Key1 => Some(1),
            Self::Key2 => Some(2),
            Self::Key3 => Some(3),
            Self::Key4 => Some(4),
            Self::Key5 => Some(5),
            Self::Key6 => Some(6),
            Self::Key7 => Some(7),
            Self::Key8 => Some(8),
            Self::Key9 => Some(9),
            _ => None,
        }
    }
}
