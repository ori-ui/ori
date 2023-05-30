use crate::Modifiers;

#[derive(Clone, Debug, Default)]
pub struct KeyboardEvent {
    pub pressed: bool,
    pub key: Option<Key>,
    pub text: Option<String>,
    pub modifiers: Modifiers,
}

impl KeyboardEvent {
    pub fn is_pressed(&self, key: Key) -> bool {
        self.pressed && self.key == Some(key)
    }

    pub fn is_released(&self, key: Key) -> bool {
        !self.pressed && self.key == Some(key)
    }

    pub fn is_press(&self) -> bool {
        self.pressed && self.key.is_some()
    }

    pub fn is_release(&self) -> bool {
        !self.pressed && self.key.is_some()
    }
}

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
