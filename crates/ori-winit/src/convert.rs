use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hash, Hasher},
};

use ori_core::{Cursor, Key, PointerButton};
use winit::event::{DeviceId, ElementState, MouseButton, VirtualKeyCode};

pub(crate) fn convert_device_id(device_id: DeviceId) -> u64 {
    let mut hasher = RandomState::new().build_hasher();
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
        VirtualKeyCode::Key0 => Key::Key0,
        VirtualKeyCode::Key1 => Key::Key1,
        VirtualKeyCode::Key2 => Key::Key2,
        VirtualKeyCode::Key3 => Key::Key3,
        VirtualKeyCode::Key4 => Key::Key4,
        VirtualKeyCode::Key5 => Key::Key5,
        VirtualKeyCode::Key6 => Key::Key6,
        VirtualKeyCode::Key7 => Key::Key7,
        VirtualKeyCode::Key8 => Key::Key8,
        VirtualKeyCode::Key9 => Key::Key9,
        VirtualKeyCode::Numpad0 => Key::Num0,
        VirtualKeyCode::Numpad1 => Key::Num1,
        VirtualKeyCode::Numpad2 => Key::Num2,
        VirtualKeyCode::Numpad3 => Key::Num3,
        VirtualKeyCode::Numpad4 => Key::Num4,
        VirtualKeyCode::Numpad5 => Key::Num5,
        VirtualKeyCode::Numpad6 => Key::Num6,
        VirtualKeyCode::Numpad7 => Key::Num7,
        VirtualKeyCode::Numpad8 => Key::Num8,
        VirtualKeyCode::Numpad9 => Key::Num9,
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
        VirtualKeyCode::LShift => Key::LeftShift,
        VirtualKeyCode::RShift => Key::RightShift,
        VirtualKeyCode::LControl => Key::LeftCtrl,
        VirtualKeyCode::RControl => Key::RightCtrl,
        VirtualKeyCode::LAlt => Key::LeftAlt,
        VirtualKeyCode::RAlt => Key::RightAlt,
        VirtualKeyCode::LWin => Key::Menu,
        VirtualKeyCode::RWin => Key::Menu,
        VirtualKeyCode::Up => Key::Up,
        VirtualKeyCode::Down => Key::Down,
        VirtualKeyCode::Left => Key::Left,
        VirtualKeyCode::Right => Key::Right,
        _ => return None,
    })
}

pub(crate) fn convert_cursor_icon(cursor_icon: Cursor) -> winit::window::CursorIcon {
    match cursor_icon {
        Cursor::Default => winit::window::CursorIcon::Default,
        Cursor::Crosshair => winit::window::CursorIcon::Crosshair,
        Cursor::Pointer => winit::window::CursorIcon::Hand,
        Cursor::Arrow => winit::window::CursorIcon::Arrow,
        Cursor::Move => winit::window::CursorIcon::Move,
        Cursor::Text => winit::window::CursorIcon::Text,
        Cursor::Wait => winit::window::CursorIcon::Wait,
        Cursor::Help => winit::window::CursorIcon::Help,
        Cursor::Progress => winit::window::CursorIcon::Progress,
        Cursor::NotAllowed => winit::window::CursorIcon::NotAllowed,
        Cursor::ContextMenu => winit::window::CursorIcon::ContextMenu,
        Cursor::Cell => winit::window::CursorIcon::Cell,
        Cursor::VerticalText => winit::window::CursorIcon::VerticalText,
        Cursor::Alias => winit::window::CursorIcon::Alias,
        Cursor::Copy => winit::window::CursorIcon::Copy,
        Cursor::NoDrop => winit::window::CursorIcon::NoDrop,
        Cursor::Grab => winit::window::CursorIcon::Grab,
        Cursor::Grabbing => winit::window::CursorIcon::Grabbing,
        Cursor::AllScroll => winit::window::CursorIcon::AllScroll,
        Cursor::ZoomIn => winit::window::CursorIcon::ZoomIn,
        Cursor::ZoomOut => winit::window::CursorIcon::ZoomOut,
        Cursor::EResize => winit::window::CursorIcon::EResize,
        Cursor::NResize => winit::window::CursorIcon::NResize,
        Cursor::NeResize => winit::window::CursorIcon::NeResize,
        Cursor::NwResize => winit::window::CursorIcon::NwResize,
        Cursor::SResize => winit::window::CursorIcon::SResize,
        Cursor::SeResize => winit::window::CursorIcon::SeResize,
        Cursor::SwResize => winit::window::CursorIcon::SwResize,
        Cursor::WResize => winit::window::CursorIcon::WResize,
        Cursor::EwResize => winit::window::CursorIcon::EwResize,
        Cursor::NsResize => winit::window::CursorIcon::NsResize,
        Cursor::NeswResize => winit::window::CursorIcon::NeswResize,
        Cursor::NwseResize => winit::window::CursorIcon::NwseResize,
        Cursor::ColResize => winit::window::CursorIcon::ColResize,
        Cursor::RowResize => winit::window::CursorIcon::RowResize,
    }
}
