use baseview::{MouseButton, MouseCursor, Point, ScrollDelta};
use keyboard_types::KeyState;
use ori_core::{Cursor, Key, Modifiers, PointerButton};
use ori_graphics::Vec2;

pub(crate) fn convert_modifiers(modifiers: keyboard_types::Modifiers) -> Modifiers {
    Modifiers {
        shift: modifiers.contains(keyboard_types::Modifiers::SHIFT),
        ctrl: modifiers.contains(keyboard_types::Modifiers::CONTROL),
        alt: modifiers.contains(keyboard_types::Modifiers::ALT),
        meta: modifiers.contains(keyboard_types::Modifiers::META),
    }
}

pub(crate) fn convert_scroll_delta(delta: ScrollDelta) -> Vec2 {
    match delta {
        ScrollDelta::Lines { x, y } => Vec2::new(x as f32, y as f32),
        ScrollDelta::Pixels { x, y } => Vec2::new(x, y),
    }
}

pub(crate) fn convert_point(point: Point) -> Vec2 {
    Vec2::new(point.x as f32, point.y as f32)
}

pub(crate) fn is_pressed(state: KeyState) -> bool {
    match state {
        KeyState::Down => true,
        KeyState::Up => false,
    }
}

pub(crate) fn convert_mouse_button(button: MouseButton) -> PointerButton {
    match button {
        MouseButton::Left => PointerButton::Primary,
        MouseButton::Right => PointerButton::Secondary,
        MouseButton::Middle => PointerButton::Tertiary,
        MouseButton::Back => PointerButton::Other(4),
        MouseButton::Forward => PointerButton::Other(5),
        MouseButton::Other(other) => PointerButton::Other(other as u16),
    }
}

pub(crate) fn convert_key(key: keyboard_types::Key) -> Option<Key> {
    Some(match key {
        // Alphabetical keys
        keyboard_types::Key::Character(c) => match c.as_str() {
            "A" | "a" => Key::A,
            "B" | "b" => Key::B,
            "C" | "c" => Key::C,
            "D" | "d" => Key::D,
            "E" | "e" => Key::E,
            "F" | "f" => Key::F,
            "G" | "g" => Key::G,
            "H" | "h" => Key::H,
            "I" | "i" => Key::I,
            "J" | "j" => Key::J,
            "K" | "k" => Key::K,
            "L" | "l" => Key::L,
            "M" | "m" => Key::M,
            "N" | "n" => Key::N,
            "O" | "o" => Key::O,
            "P" | "p" => Key::P,
            "Q" | "q" => Key::Q,
            "R" | "r" => Key::R,
            "S" | "s" => Key::S,
            "T" | "t" => Key::T,
            "U" | "u" => Key::U,
            "V" | "v" => Key::V,
            "W" | "w" => Key::W,
            "X" | "x" => Key::X,
            "Y" | "y" => Key::Y,
            "Z" | "z" => Key::Z,

            // Number keys
            "0" => Key::Key0,
            "1" => Key::Key1,
            "2" => Key::Key2,
            "3" => Key::Key3,
            "4" => Key::Key4,
            "5" => Key::Key5,
            "6" => Key::Key6,
            "7" => Key::Key7,
            "8" => Key::Key8,
            "9" => Key::Key9,

            // Space key
            " " => Key::Space,

            // Symbol keys
            "+" | "=" => Key::Plus,
            "-" | "_" => Key::Minus,
            "." => Key::Period,
            "," => Key::Comma,

            _ => return None,
        },
        // Function keys
        keyboard_types::Key::F1 => Key::F1,
        keyboard_types::Key::F2 => Key::F2,
        keyboard_types::Key::F3 => Key::F3,
        keyboard_types::Key::F4 => Key::F4,
        keyboard_types::Key::F5 => Key::F5,
        keyboard_types::Key::F6 => Key::F6,
        keyboard_types::Key::F7 => Key::F7,
        keyboard_types::Key::F8 => Key::F8,
        keyboard_types::Key::F9 => Key::F9,
        keyboard_types::Key::F10 => Key::F10,
        keyboard_types::Key::F11 => Key::F11,
        keyboard_types::Key::F12 => Key::F12,
        // Arrow keys
        keyboard_types::Key::ArrowUp => Key::Up,
        keyboard_types::Key::ArrowDown => Key::Down,
        keyboard_types::Key::ArrowLeft => Key::Left,
        keyboard_types::Key::ArrowRight => Key::Right,
        // Special keys
        keyboard_types::Key::Escape => Key::Escape,
        keyboard_types::Key::Insert => Key::Insert,
        keyboard_types::Key::Delete => Key::Delete,
        keyboard_types::Key::Home => Key::Home,
        keyboard_types::Key::End => Key::End,
        keyboard_types::Key::PageUp => Key::PageUp,
        keyboard_types::Key::PageDown => Key::PageDown,
        keyboard_types::Key::Backspace => Key::Backspace,
        keyboard_types::Key::Tab => Key::Tab,
        keyboard_types::Key::Enter => Key::Enter,
        // Modifier keys
        keyboard_types::Key::Shift => Key::Shift,
        keyboard_types::Key::Control => Key::Ctrl,
        keyboard_types::Key::Alt => Key::Alt,
        keyboard_types::Key::Meta => Key::Meta,
        _ => return None,
    })
}

#[allow(dead_code)]
pub(crate) fn convert_cursor_icon(cursor_icon: Cursor) -> MouseCursor {
    match cursor_icon {
        Cursor::Default => MouseCursor::Default,
        Cursor::Crosshair => MouseCursor::Crosshair,
        Cursor::Pointer => MouseCursor::Hand,
        Cursor::Arrow => MouseCursor::Default,
        Cursor::Move => MouseCursor::Move,
        Cursor::Text => MouseCursor::Text,
        Cursor::Wait => MouseCursor::Working,
        Cursor::Help => MouseCursor::Help,
        Cursor::Progress => MouseCursor::PtrWorking,
        Cursor::NotAllowed => MouseCursor::NotAllowed,
        Cursor::ContextMenu => MouseCursor::Default,
        Cursor::Cell => MouseCursor::Cell,
        Cursor::VerticalText => MouseCursor::VerticalText,
        Cursor::Alias => MouseCursor::Alias,
        Cursor::Copy => MouseCursor::Copy,
        Cursor::NoDrop => MouseCursor::NotAllowed,
        Cursor::Grab => MouseCursor::Hand,
        Cursor::Grabbing => MouseCursor::HandGrabbing,
        Cursor::AllScroll => MouseCursor::AllScroll,
        Cursor::ZoomIn => MouseCursor::ZoomIn,
        Cursor::ZoomOut => MouseCursor::ZoomOut,
        Cursor::EResize => MouseCursor::EResize,
        Cursor::NResize => MouseCursor::NResize,
        Cursor::NeResize => MouseCursor::NeResize,
        Cursor::NwResize => MouseCursor::NwResize,
        Cursor::SResize => MouseCursor::SResize,
        Cursor::SeResize => MouseCursor::SeResize,
        Cursor::SwResize => MouseCursor::SwResize,
        Cursor::WResize => MouseCursor::WResize,
        Cursor::EwResize => MouseCursor::EwResize,
        Cursor::NsResize => MouseCursor::NsResize,
        Cursor::NeswResize => MouseCursor::NeswResize,
        Cursor::NwseResize => MouseCursor::NwseResize,
        Cursor::ColResize => MouseCursor::ColResize,
        Cursor::RowResize => MouseCursor::RowResize,
    }
}
