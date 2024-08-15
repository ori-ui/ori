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
    Minus,
    Equal,
    BracketLeft,
    BracketRight,
    Semicolon,
    Apostrophe,
    Backtick,
    Backslash,
    Comma,
    Period,
    Slash,
    NumStar,

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
    LShift,
    LCtrl,
    LAlt,
    LMeta,
    RShift,
    RCtrl,
    RAlt,
    RMeta,
}

impl Code {
    /// Convert a Linux scancode to a key code.
    pub fn from_linux_scancode(scancode: u8) -> Option<Self> {
        Some(match scancode {
            0x01 => Self::Escape,
            0x02 => Self::Key1,
            0x03 => Self::Key2,
            0x04 => Self::Key3,
            0x05 => Self::Key4,
            0x06 => Self::Key5,
            0x07 => Self::Key6,
            0x08 => Self::Key7,
            0x09 => Self::Key8,
            0x0a => Self::Key9,
            0x0b => Self::Key0,
            0x0c => Self::Minus,
            0x0d => Self::Equal,
            0x0e => Self::Backspace,
            0x0f => Self::Tab,
            0x10 => Self::Q,
            0x11 => Self::W,
            0x12 => Self::E,
            0x13 => Self::R,
            0x14 => Self::T,
            0x15 => Self::Y,
            0x16 => Self::U,
            0x17 => Self::I,
            0x18 => Self::O,
            0x19 => Self::P,
            0x1a => Self::BracketLeft,
            0x1b => Self::BracketRight,
            0x1c => Self::Enter,
            0x1d => Self::LCtrl,
            0x1e => Self::A,
            0x1f => Self::S,
            0x20 => Self::D,
            0x21 => Self::F,
            0x22 => Self::G,
            0x23 => Self::H,
            0x24 => Self::J,
            0x25 => Self::K,
            0x26 => Self::L,
            0x27 => Self::Semicolon,
            0x28 => Self::Apostrophe,
            0x29 => Self::Backtick,
            0x2a => Self::LShift,
            0x2b => Self::Backslash,
            0x2c => Self::Z,
            0x2d => Self::X,
            0x2e => Self::C,
            0x2f => Self::V,
            0x30 => Self::B,
            0x31 => Self::N,
            0x32 => Self::M,
            0x33 => Self::Comma,
            0x34 => Self::Period,
            0x35 => Self::Slash,
            0x36 => Self::RShift,
            0x37 => Self::NumStar,
            0x38 => Self::LAlt,
            0x39 => Self::Space,
            0x3a => Self::CapsLock,
            0x3b => Self::F1,
            0x3c => Self::F2,
            0x3d => Self::F3,
            0x3e => Self::F4,
            0x3f => Self::F5,
            0x40 => Self::F6,
            0x41 => Self::F7,
            0x42 => Self::F8,
            0x43 => Self::F9,
            0x44 => Self::F10,

            _ => return None,
        })
    }

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
