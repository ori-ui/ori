use ori_core::{Code, Cursor, PointerButton};
use winit::{
    event::{ElementState, MouseButton, VirtualKeyCode},
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
        MouseButton::Other(other) => PointerButton::Other(other),
    }
}

pub(crate) fn convert_key(key: VirtualKeyCode) -> Option<Code> {
    Some(match key {
        // Alphabetical keys
        VirtualKeyCode::A => Code::A,
        VirtualKeyCode::B => Code::B,
        VirtualKeyCode::C => Code::C,
        VirtualKeyCode::D => Code::D,
        VirtualKeyCode::E => Code::E,
        VirtualKeyCode::F => Code::F,
        VirtualKeyCode::G => Code::G,
        VirtualKeyCode::H => Code::H,
        VirtualKeyCode::I => Code::I,
        VirtualKeyCode::J => Code::J,
        VirtualKeyCode::K => Code::K,
        VirtualKeyCode::L => Code::L,
        VirtualKeyCode::M => Code::M,
        VirtualKeyCode::N => Code::N,
        VirtualKeyCode::O => Code::O,
        VirtualKeyCode::P => Code::P,
        VirtualKeyCode::Q => Code::Q,
        VirtualKeyCode::R => Code::R,
        VirtualKeyCode::S => Code::S,
        VirtualKeyCode::T => Code::T,
        VirtualKeyCode::U => Code::U,
        VirtualKeyCode::V => Code::V,
        VirtualKeyCode::W => Code::W,
        VirtualKeyCode::X => Code::X,
        VirtualKeyCode::Y => Code::Y,
        VirtualKeyCode::Z => Code::Z,
        // Number keys
        VirtualKeyCode::Key0 | VirtualKeyCode::Numpad0 => Code::Key0,
        VirtualKeyCode::Key1 | VirtualKeyCode::Numpad1 => Code::Key1,
        VirtualKeyCode::Key2 | VirtualKeyCode::Numpad2 => Code::Key2,
        VirtualKeyCode::Key3 | VirtualKeyCode::Numpad3 => Code::Key3,
        VirtualKeyCode::Key4 | VirtualKeyCode::Numpad4 => Code::Key4,
        VirtualKeyCode::Key5 | VirtualKeyCode::Numpad5 => Code::Key5,
        VirtualKeyCode::Key6 | VirtualKeyCode::Numpad6 => Code::Key6,
        VirtualKeyCode::Key7 | VirtualKeyCode::Numpad7 => Code::Key7,
        VirtualKeyCode::Key8 | VirtualKeyCode::Numpad8 => Code::Key8,
        VirtualKeyCode::Key9 | VirtualKeyCode::Numpad9 => Code::Key9,
        // Function keys
        VirtualKeyCode::F1 => Code::F1,
        VirtualKeyCode::F2 => Code::F2,
        VirtualKeyCode::F3 => Code::F3,
        VirtualKeyCode::F4 => Code::F4,
        VirtualKeyCode::F5 => Code::F5,
        VirtualKeyCode::F6 => Code::F6,
        VirtualKeyCode::F7 => Code::F7,
        VirtualKeyCode::F8 => Code::F8,
        VirtualKeyCode::F9 => Code::F9,
        VirtualKeyCode::F10 => Code::F10,
        VirtualKeyCode::F11 => Code::F11,
        VirtualKeyCode::F12 => Code::F12,
        // Symbol keys
        VirtualKeyCode::Equals | VirtualKeyCode::NumpadAdd | VirtualKeyCode::Plus => Code::Plus,
        VirtualKeyCode::Minus | VirtualKeyCode::NumpadSubtract => Code::Minus,
        VirtualKeyCode::Asterisk | VirtualKeyCode::NumpadMultiply => Code::Asterisk,
        VirtualKeyCode::Slash | VirtualKeyCode::NumpadDivide => Code::Slash,
        VirtualKeyCode::Backslash => Code::Backslash,
        VirtualKeyCode::Period | VirtualKeyCode::NumpadDecimal => Code::Period,
        VirtualKeyCode::Comma | VirtualKeyCode::NumpadComma => Code::Comma,
        // Arrow keys
        VirtualKeyCode::Up => Code::Up,
        VirtualKeyCode::Down => Code::Down,
        VirtualKeyCode::Left => Code::Left,
        VirtualKeyCode::Right => Code::Right,
        // Special keys
        VirtualKeyCode::Escape => Code::Escape,
        VirtualKeyCode::Insert => Code::Insert,
        VirtualKeyCode::Delete => Code::Delete,
        VirtualKeyCode::Home => Code::Home,
        VirtualKeyCode::End => Code::End,
        VirtualKeyCode::PageUp => Code::PageUp,
        VirtualKeyCode::PageDown => Code::PageDown,
        VirtualKeyCode::Back => Code::Backspace,
        VirtualKeyCode::Tab => Code::Tab,
        VirtualKeyCode::Return | VirtualKeyCode::NumpadEnter => Code::Enter,
        VirtualKeyCode::Space => Code::Space,
        // Modifier keys
        VirtualKeyCode::LShift | VirtualKeyCode::RShift => Code::Shift,
        VirtualKeyCode::LControl | VirtualKeyCode::RControl => Code::Ctrl,
        VirtualKeyCode::LAlt | VirtualKeyCode::RAlt => Code::Alt,
        VirtualKeyCode::LWin | VirtualKeyCode::RWin => Code::Meta,
        _ => return None,
    })
}

#[allow(unused)]
pub(crate) fn convert_cursor_icon(cursor_icon: Cursor) -> CursorIcon {
    match cursor_icon {
        Cursor::Default => CursorIcon::Default,
        Cursor::Crosshair => CursorIcon::Crosshair,
        Cursor::Pointer => CursorIcon::Hand,
        Cursor::Arrow => CursorIcon::Arrow,
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
