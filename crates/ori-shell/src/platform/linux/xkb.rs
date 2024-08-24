use ori_core::event::Key;
use xkeysym::Keysym;

pub fn keysym_to_key(keysym: Keysym) -> Key {
    match keysym {
        /* modifier keys */
        Keysym::Alt_L | Keysym::Alt_R => Key::Alt,
        Keysym::SUN_AltGraph | Keysym::ISO_Level3_Shift => Key::AltGraph,
        Keysym::Control_L | Keysym::Control_R => Key::Control,
        Keysym::Shift_L | Keysym::Shift_R => Key::Shift,
        Keysym::Meta_L | Keysym::Meta_R => Key::Meta,
        Keysym::Super_L | Keysym::Super_R => Key::Super,
        Keysym::Hyper_L | Keysym::Hyper_R => Key::Hyper,
        Keysym::XF86_Fn | Keysym::XF86_Launch1 => Key::Fn,

        /* lock keys */
        Keysym::Num_Lock => Key::NumLock,
        Keysym::Caps_Lock => Key::CapsLock,
        Keysym::Scroll_Lock => Key::ScrollLock,

        /* function keys */
        Keysym::F1 => Key::F1,
        Keysym::F2 => Key::F2,
        Keysym::F3 => Key::F3,
        Keysym::F4 => Key::F4,
        Keysym::F5 => Key::F5,
        Keysym::F6 => Key::F6,
        Keysym::F7 => Key::F7,
        Keysym::F8 => Key::F8,
        Keysym::F9 => Key::F9,
        Keysym::F10 => Key::F10,
        Keysym::F11 => Key::F11,
        Keysym::F12 => Key::F12,
        Keysym::F13 => Key::F13,
        Keysym::F14 => Key::F14,
        Keysym::F15 => Key::F15,
        Keysym::F16 => Key::F16,
        Keysym::F17 => Key::F17,
        Keysym::F18 => Key::F18,
        Keysym::F19 => Key::F19,
        Keysym::F20 => Key::F20,
        Keysym::F21 => Key::F21,
        Keysym::F22 => Key::F22,
        Keysym::F23 => Key::F23,
        Keysym::F24 => Key::F24,

        /* misc keys */
        Keysym::Return | Keysym::KP_Enter => Key::Enter,
        Keysym::Tab => Key::Tab,
        Keysym::Down => Key::Down,
        Keysym::Left => Key::Left,
        Keysym::Right => Key::Right,
        Keysym::Up => Key::Up,
        Keysym::End => Key::End,
        Keysym::Home => Key::Home,
        Keysym::Page_Down => Key::PageDown,
        Keysym::Page_Up => Key::PageUp,
        Keysym::BackSpace => Key::Backspace,
        Keysym::Clear => Key::Clear,
        Keysym::SUN_Copy | Keysym::XF86_Copy => Key::Copy,
        Keysym::SUN_Cut | Keysym::XF86_Cut => Key::Cut,
        Keysym::Delete => Key::Delete,
        Keysym::Insert => Key::Insert,
        Keysym::SUN_Paste | Keysym::OSF_Paste | Keysym::XF86_Paste => Key::Paste,
        Keysym::Cancel => Key::Cancel,
        Keysym::Escape => Key::Escape,
        Keysym::Execute => Key::Execute,
        Keysym::Find => Key::Find,
        Keysym::Help => Key::Help,
        Keysym::Pause => Key::Pause,
        Keysym::Select => Key::Select,
        Keysym::SUN_Print_Screen => Key::PrintScreen,
        Keysym::Codeinput => Key::CodeInput,

        _ => Key::Unidentified,
    }
}
