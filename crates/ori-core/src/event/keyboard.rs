use super::Modifiers;

/// A trait for checking if something is a certain key.
pub trait IsKey {
    /// Check if the key is the given key.
    fn is(&self, key: Key, code: Option<Code>) -> bool;
}

impl IsKey for char {
    fn is(&self, key: Key, _: Option<Code>) -> bool {
        Key::Character(*self) == key
    }
}

impl IsKey for Key {
    fn is(&self, key: Key, _: Option<Code>) -> bool {
        *self == key
    }
}

impl IsKey for Code {
    fn is(&self, _: Key, code: Option<Code>) -> bool {
        Some(self) == code.as_ref()
    }
}

impl<T: IsKey> IsKey for &T {
    fn is(&self, key: Key, code: Option<Code>) -> bool {
        T::is(self, key, code)
    }
}

impl<T: IsKey> IsKey for &mut T {
    fn is(&self, key: Key, code: Option<Code>) -> bool {
        T::is(self, key, code)
    }
}

/// An event fired when a key is pressed.
#[derive(Clone, Debug)]
pub struct KeyPressed {
    /// The key that was pressed or released.
    pub key: Key,

    /// The code of the key that was pressed or released.
    pub code: Option<Code>,

    /// The text that was entered.
    pub text: Option<String>,

    /// The modifiers that were active.
    pub modifiers: Modifiers,
}

impl KeyPressed {
    /// Check if the `key` is pressed.
    pub fn is_key(&self, key: impl IsKey) -> bool {
        key.is(self.key, self.code)
    }
}

/// An event fired when a key is released.
#[derive(Clone, Debug)]
pub struct KeyReleased {
    /// The key that was pressed or released.
    pub key: Key,

    /// The code of the key that was pressed or released.
    pub code: Option<Code>,

    /// The modifiers that were active.
    pub modifiers: Modifiers,
}

impl KeyReleased {
    /// Check if the `key` is released.
    pub fn is_key(&self, key: impl IsKey) -> bool {
        key.is(self.key, self.code)
    }
}

/// A keyboard key.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Key {
    /* special keys */
    Character(char),
    Unidentified,

    /* modifier keys */
    Alt,
    AltGraph,
    Control,
    Shift,
    Meta,
    Super,
    Hyper,
    Symbol,
    Fn,

    /* lock keys */
    FnLock,
    NumLock,
    CapsLock,
    ScrollLock,
    SymbolLock,

    /* function keys */
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
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    /* misc keys */
    Enter,
    Tab,
    Down,
    Left,
    Right,
    Up,
    End,
    Home,
    PageDown,
    PageUp,
    Backspace,
    Clear,
    Copy,
    Cut,
    Delete,
    Insert,
    Paste,
    Redo,
    Undo,
    Accept,
    Again,
    Cancel,
    Escape,
    Execute,
    Find,
    Help,
    Pause,
    Play,
    Select,
    PrintScreen,
    Alphanumeric,
    CodeInput,
    Compose,
    Convert,
    Dead,
    HangulMode,
    HanjaMode,
    JunjaMode,
    KanjiMode,
    KanaMode,
    Eisu,
    Hankaku,
    Hiragana,
    Katakana,
    HiraganaKatakana,
    Romaji,
    Zenkaku,
    ZenkakuHankaku,
}

/// A keyboard key-code.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    // Numpad keys
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,

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
    NumLock,
    ScrollLock,

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
            0x45 => Self::NumLock,
            0x46 => Self::ScrollLock,
            0x47 => Self::Home,

            0x67 => Self::Up,
            0x69 => Self::Left,
            0x6a => Self::Right,
            0x6c => Self::Down,

            _ => return None,
        })
    }
}
