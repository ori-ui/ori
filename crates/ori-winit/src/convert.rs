use std::hash::{Hash, Hasher};

use ori_core::{Cursor, Key, PointerButton};
use winit::{
    event::{DeviceId, ElementState, MouseButton, VirtualKeyCode},
    window::CursorIcon,
};

pub(crate) fn convert_device_id(device_id: DeviceId) -> u64 {
    let mut hasher = seahash::SeaHasher::new();
    device_id.hash(&mut hasher);
    hasher.finish()
}

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

pub(crate) fn convert_key(key: VirtualKeyCode) -> Option<Key> {
    Some(match key {
        // Alphabetical keys
        VirtualKeyCode::A => Key::A,
        VirtualKeyCode::B => Key::B,
        VirtualKeyCode::C => Key::C,
        VirtualKeyCode::D => Key::D,
        VirtualKeyCode::E => Key::E,
        VirtualKeyCode::F => Key::F,
        VirtualKeyCode::G => Key::G,
        VirtualKeyCode::H => Key::H,
        VirtualKeyCode::I => Key::I,
        VirtualKeyCode::J => Key::J,
        VirtualKeyCode::K => Key::K,
        VirtualKeyCode::L => Key::L,
        VirtualKeyCode::M => Key::M,
        VirtualKeyCode::N => Key::N,
        VirtualKeyCode::O => Key::O,
        VirtualKeyCode::P => Key::P,
        VirtualKeyCode::Q => Key::Q,
        VirtualKeyCode::R => Key::R,
        VirtualKeyCode::S => Key::S,
        VirtualKeyCode::T => Key::T,
        VirtualKeyCode::U => Key::U,
        VirtualKeyCode::V => Key::V,
        VirtualKeyCode::W => Key::W,
        VirtualKeyCode::X => Key::X,
        VirtualKeyCode::Y => Key::Y,
        VirtualKeyCode::Z => Key::Z,
        // Number keys
        VirtualKeyCode::Key0 | VirtualKeyCode::Numpad0 => Key::Key0,
        VirtualKeyCode::Key1 | VirtualKeyCode::Numpad1 => Key::Key1,
        VirtualKeyCode::Key2 | VirtualKeyCode::Numpad2 => Key::Key2,
        VirtualKeyCode::Key3 | VirtualKeyCode::Numpad3 => Key::Key3,
        VirtualKeyCode::Key4 | VirtualKeyCode::Numpad4 => Key::Key4,
        VirtualKeyCode::Key5 | VirtualKeyCode::Numpad5 => Key::Key5,
        VirtualKeyCode::Key6 | VirtualKeyCode::Numpad6 => Key::Key6,
        VirtualKeyCode::Key7 | VirtualKeyCode::Numpad7 => Key::Key7,
        VirtualKeyCode::Key8 | VirtualKeyCode::Numpad8 => Key::Key8,
        VirtualKeyCode::Key9 | VirtualKeyCode::Numpad9 => Key::Key9,
        // Function keys
        VirtualKeyCode::F1 => Key::F1,
        VirtualKeyCode::F2 => Key::F2,
        VirtualKeyCode::F3 => Key::F3,
        VirtualKeyCode::F4 => Key::F4,
        VirtualKeyCode::F5 => Key::F5,
        VirtualKeyCode::F6 => Key::F6,
        VirtualKeyCode::F7 => Key::F7,
        VirtualKeyCode::F8 => Key::F8,
        VirtualKeyCode::F9 => Key::F9,
        VirtualKeyCode::F10 => Key::F10,
        VirtualKeyCode::F11 => Key::F11,
        VirtualKeyCode::F12 => Key::F12,
        // Symbol keys
        VirtualKeyCode::Equals | VirtualKeyCode::NumpadAdd | VirtualKeyCode::Plus => Key::Plus,
        VirtualKeyCode::Minus | VirtualKeyCode::NumpadSubtract => Key::Minus,
        VirtualKeyCode::Period | VirtualKeyCode::NumpadDecimal => Key::Period,
        VirtualKeyCode::Comma | VirtualKeyCode::NumpadComma => Key::Comma,
        // Arrow keys
        VirtualKeyCode::Up => Key::Up,
        VirtualKeyCode::Down => Key::Down,
        VirtualKeyCode::Left => Key::Left,
        VirtualKeyCode::Right => Key::Right,
        // Special keys
        VirtualKeyCode::Escape => Key::Escape,
        VirtualKeyCode::Insert => Key::Insert,
        VirtualKeyCode::Delete => Key::Delete,
        VirtualKeyCode::Home => Key::Home,
        VirtualKeyCode::End => Key::End,
        VirtualKeyCode::PageUp => Key::PageUp,
        VirtualKeyCode::PageDown => Key::PageDown,
        VirtualKeyCode::Back => Key::Backspace,
        VirtualKeyCode::Tab => Key::Tab,
        VirtualKeyCode::Return => Key::Enter,
        VirtualKeyCode::Space => Key::Space,
        // Modifier keys
        VirtualKeyCode::LShift | VirtualKeyCode::RShift => Key::Shift,
        VirtualKeyCode::LControl | VirtualKeyCode::RControl => Key::Ctrl,
        VirtualKeyCode::LAlt | VirtualKeyCode::RAlt => Key::Alt,
        VirtualKeyCode::LWin | VirtualKeyCode::RWin => Key::Meta,
        _ => return None,
    })
}

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
