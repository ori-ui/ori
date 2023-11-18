use ori_core::{
    event::{Code, PointerButton},
    window::Cursor,
};
use winit::{
    event::{ElementState, MouseButton},
    keyboard::KeyCode,
    window::CursorIcon,
};

pub(crate) fn is_pressed(state: ElementState) -> bool {
    match state {
        ElementState::Pressed => true,
        ElementState::Released => false,
    }
}

pub(crate) fn convert_mouse_button(button: MouseButton) -> PointerButton {
    match button {
        MouseButton::Left => PointerButton::Primary,
        MouseButton::Right => PointerButton::Secondary,
        MouseButton::Middle => PointerButton::Tertiary,
        MouseButton::Back => PointerButton::Back,
        MouseButton::Forward => PointerButton::Forward,
        MouseButton::Other(other) => PointerButton::Other(other),
    }
}

pub(crate) fn convert_key(key: KeyCode) -> Option<Code> {
    Some(match key {
        // Alphabetical keys
        KeyCode::KeyA => Code::A,
        KeyCode::KeyB => Code::B,
        KeyCode::KeyC => Code::C,
        KeyCode::KeyD => Code::D,
        KeyCode::KeyE => Code::E,
        KeyCode::KeyF => Code::F,
        KeyCode::KeyG => Code::G,
        KeyCode::KeyH => Code::H,
        KeyCode::KeyI => Code::I,
        KeyCode::KeyJ => Code::J,
        KeyCode::KeyK => Code::K,
        KeyCode::KeyL => Code::L,
        KeyCode::KeyM => Code::M,
        KeyCode::KeyN => Code::N,
        KeyCode::KeyO => Code::O,
        KeyCode::KeyP => Code::P,
        KeyCode::KeyQ => Code::Q,
        KeyCode::KeyR => Code::R,
        KeyCode::KeyS => Code::S,
        KeyCode::KeyT => Code::T,
        KeyCode::KeyU => Code::U,
        KeyCode::KeyV => Code::V,
        KeyCode::KeyW => Code::W,
        KeyCode::KeyX => Code::X,
        KeyCode::KeyY => Code::Y,
        KeyCode::KeyZ => Code::Z,
        // Number keys
        KeyCode::Digit0 | KeyCode::Numpad0 => Code::Key0,
        KeyCode::Digit1 | KeyCode::Numpad1 => Code::Key1,
        KeyCode::Digit2 | KeyCode::Numpad2 => Code::Key2,
        KeyCode::Digit3 | KeyCode::Numpad3 => Code::Key3,
        KeyCode::Digit4 | KeyCode::Numpad4 => Code::Key4,
        KeyCode::Digit5 | KeyCode::Numpad5 => Code::Key5,
        KeyCode::Digit6 | KeyCode::Numpad6 => Code::Key6,
        KeyCode::Digit7 | KeyCode::Numpad7 => Code::Key7,
        KeyCode::Digit8 | KeyCode::Numpad8 => Code::Key8,
        KeyCode::Digit9 | KeyCode::Numpad9 => Code::Key9,
        // Function keys
        KeyCode::F1 => Code::F1,
        KeyCode::F2 => Code::F2,
        KeyCode::F3 => Code::F3,
        KeyCode::F4 => Code::F4,
        KeyCode::F5 => Code::F5,
        KeyCode::F6 => Code::F6,
        KeyCode::F7 => Code::F7,
        KeyCode::F8 => Code::F8,
        KeyCode::F9 => Code::F9,
        KeyCode::F10 => Code::F10,
        KeyCode::F11 => Code::F11,
        KeyCode::F12 => Code::F12,
        // Symbol keys
        KeyCode::Equal | KeyCode::NumpadAdd => Code::Plus,
        KeyCode::Minus | KeyCode::NumpadSubtract => Code::Minus,
        KeyCode::NumpadStar | KeyCode::NumpadMultiply => Code::Asterisk,
        KeyCode::Slash | KeyCode::NumpadDivide => Code::Slash,
        KeyCode::Backslash => Code::Backslash,
        KeyCode::Period | KeyCode::NumpadDecimal => Code::Period,
        KeyCode::Comma | KeyCode::NumpadComma => Code::Comma,
        // Arrow keys
        KeyCode::ArrowUp => Code::Up,
        KeyCode::ArrowDown => Code::Down,
        KeyCode::ArrowLeft => Code::Left,
        KeyCode::ArrowRight => Code::Right,
        // Special keys
        KeyCode::Escape => Code::Escape,
        KeyCode::Insert => Code::Insert,
        KeyCode::Delete => Code::Delete,
        KeyCode::Home => Code::Home,
        KeyCode::End => Code::End,
        KeyCode::PageUp => Code::PageUp,
        KeyCode::PageDown => Code::PageDown,
        KeyCode::Backspace | KeyCode::NumpadBackspace => Code::Backspace,
        KeyCode::Tab => Code::Tab,
        KeyCode::Enter | KeyCode::NumpadEnter => Code::Enter,
        KeyCode::Space => Code::Space,
        // Modifier keys
        KeyCode::ShiftLeft | KeyCode::ShiftRight => Code::Shift,
        KeyCode::ControlLeft | KeyCode::ControlRight => Code::Ctrl,
        KeyCode::AltLeft | KeyCode::AltRight => Code::Alt,
        KeyCode::Meta => Code::Meta,
        _ => return None,
    })
}

#[allow(unused)]
pub(crate) fn convert_cursor_icon(cursor_icon: Cursor) -> CursorIcon {
    match cursor_icon {
        Cursor::Default => CursorIcon::Default,
        Cursor::Crosshair => CursorIcon::Crosshair,
        Cursor::Pointer => CursorIcon::Pointer,
        Cursor::Arrow => CursorIcon::Default,
        Cursor::Move => CursorIcon::Move,
        Cursor::Text => CursorIcon::Text,
        Cursor::Wait => CursorIcon::Wait,
        Cursor::Help => CursorIcon::Help,
        Cursor::Progress => CursorIcon::Progress,
        Cursor::NotAllowed => CursorIcon::NotAllowed,
        Cursor::ContextMenu => CursorIcon::ContextMenu,
        Cursor::Cell => CursorIcon::Cell,
        Cursor::VerticalText => CursorIcon::VerticalText,
        Cursor::Alias => CursorIcon::Alias,
        Cursor::Copy => CursorIcon::Copy,
        Cursor::NoDrop => CursorIcon::NoDrop,
        Cursor::Grab => CursorIcon::Grab,
        Cursor::Grabbing => CursorIcon::Grabbing,
        Cursor::AllScroll => CursorIcon::AllScroll,
        Cursor::ZoomIn => CursorIcon::ZoomIn,
        Cursor::ZoomOut => CursorIcon::ZoomOut,
        Cursor::EResize => CursorIcon::EResize,
        Cursor::NResize => CursorIcon::NResize,
        Cursor::NeResize => CursorIcon::NeResize,
        Cursor::NwResize => CursorIcon::NwResize,
        Cursor::SResize => CursorIcon::SResize,
        Cursor::SeResize => CursorIcon::SeResize,
        Cursor::SwResize => CursorIcon::SwResize,
        Cursor::WResize => CursorIcon::WResize,
        Cursor::EwResize => CursorIcon::EwResize,
        Cursor::NsResize => CursorIcon::NsResize,
        Cursor::NeswResize => CursorIcon::NeswResize,
        Cursor::NwseResize => CursorIcon::NwseResize,
        Cursor::ColResize => CursorIcon::ColResize,
        Cursor::RowResize => CursorIcon::RowResize,
    }
}
