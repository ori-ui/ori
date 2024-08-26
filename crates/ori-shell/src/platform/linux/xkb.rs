use std::{
    ffi,
    fs::File,
    os::fd::OwnedFd,
    ptr::{self, NonNull},
    rc::Rc,
    sync::LazyLock,
};

use ori_core::event::{Key, Modifiers};
use xkbcommon_dl::{
    xkb_context, xkb_context_flags::XKB_CONTEXT_NO_FLAGS, xkb_keymap,
    xkb_keymap_compile_flags::XKB_KEYMAP_COMPILE_NO_FLAGS,
    xkb_keymap_format::XKB_KEYMAP_FORMAT_TEXT_V1, xkb_state, xkb_state_component, xkbcommon_handle,
    XkbCommon, XKB_MOD_NAME_ALT, XKB_MOD_NAME_CTRL, XKB_MOD_NAME_LOGO, XKB_MOD_NAME_SHIFT,
};
use xkeysym::Keysym;

static XKB: LazyLock<&'static XkbCommon> = LazyLock::new(xkbcommon_handle);

#[cfg(x11_platform)]
static XKBX11: LazyLock<&'static xkbcommon_dl::x11::XkbCommonX11> =
    LazyLock::new(xkbcommon_dl::x11::xkbcommon_x11_handle);

pub struct XkbKeyboard {
    state: Option<XkbState>,
    keymap: Option<XkbKeymap>,
    context: XkbContext,
}

impl XkbKeyboard {
    pub fn new(context: &XkbContext) -> Option<Self> {
        Some(Self {
            state: None,
            keymap: None,
            context: context.clone(),
        })
    }

    #[cfg(x11_platform)]
    pub unsafe fn new_xcb(context: &XkbContext, xcb: *mut ffi::c_void) -> Option<Self> {
        let keymap = XkbKeymap::from_xcb(context, xcb)?;
        let state = XkbState::from_xcb(context, &keymap, xcb)?;

        Some(Self {
            state: Some(state),
            keymap: Some(keymap),
            context: context.clone(),
        })
    }

    #[cfg(wayland_platform)]
    pub fn set_keymap_from_fd(&mut self, fd: OwnedFd, size: usize) -> Option<()> {
        let keymap = XkbKeymap::from_fd(&self.context, fd, size)?;
        let state = XkbState::new(&keymap)?;

        self.state = Some(state);
        self.keymap = Some(keymap);

        Some(())
    }

    pub fn state(&self) -> Option<&XkbState> {
        self.state.as_ref()
    }

    pub fn keymap(&self) -> Option<&XkbKeymap> {
        self.keymap.as_ref()
    }

    pub fn keysym_to_utf8(&self, keysym: Keysym) -> Option<String> {
        let mut buffer = [0u8; 16];

        let written = unsafe {
            // get the keysym and write it to the buffer
            (XKB.xkb_keysym_to_utf8)(
                keysym.into(),
                // cast the buffer from [u8] to [i8]
                buffer.as_mut_ptr().cast(),
                buffer.len(),
            )
        };

        if written <= 0 {
            return None;
        }

        let len = written as usize - 1;
        Some(String::from_utf8_lossy(&buffer[..len]).to_string())
    }

    pub fn keysym_to_key(&self, keysym: Keysym) -> Key {
        if let Some(utf8) = self.keysym_to_utf8(keysym) {
            let mut chars = utf8.chars();
            let c = chars.next().unwrap();
            debug_assert!(chars.next().is_none());

            if !c.is_control() {
                return Key::Character(c);
            }
        }

        crate::platform::linux::xkb::keysym_to_key(keysym)
    }
}

pub struct XkbKeymap {
    keymap: NonNull<xkb_keymap>,
}

impl XkbKeymap {
    #[cfg(wayland_platform)]
    fn from_fd(context: &XkbContext, fd: OwnedFd, size: usize) -> Option<Self> {
        use memmap::MmapOptions;

        let file = File::from(fd);
        let data = unsafe { MmapOptions::new().len(size).map_copy(&file).ok()? };

        let keymap = unsafe {
            (XKB.xkb_keymap_new_from_string)(
                context.ptr(),
                data.as_ptr() as *const _,
                XKB_KEYMAP_FORMAT_TEXT_V1,
                XKB_KEYMAP_COMPILE_NO_FLAGS,
            )
        };

        NonNull::new(keymap).map(|keymap| Self { keymap })
    }

    #[cfg(x11_platform)]
    fn from_xcb(context: &XkbContext, xcb: *mut ffi::c_void) -> Option<Self> {
        let keymap = unsafe {
            (XKBX11.xkb_x11_keymap_new_from_device)(
                context.ptr(),
                xcb,
                context.core_keyboard,
                XKB_KEYMAP_COMPILE_NO_FLAGS,
            )
        };

        let keymap = NonNull::new(keymap)?;
        Some(Self { keymap })
    }

    pub fn first_keysym(&self, layout: u32, keycode: u32) -> Option<Keysym> {
        unsafe {
            let mut keysyms = ptr::null();
            let count = (XKB.xkb_keymap_key_get_syms_by_level)(
                self.keymap.as_ptr(),
                keycode,
                layout,
                0,
                &mut keysyms,
            );

            if count > 0 {
                Some(Keysym::new(*keysyms))
            } else {
                None
            }
        }
    }

    pub fn key_repeats(&self, keycode: u32) -> bool {
        unsafe { (XKB.xkb_keymap_key_repeats)(self.keymap.as_ptr(), keycode) > 0 }
    }
}

impl Drop for XkbKeymap {
    fn drop(&mut self) {
        unsafe { (XKB.xkb_keymap_unref)(self.keymap.as_ptr()) }
    }
}

pub struct XkbState {
    state: NonNull<xkb_state>,
}

impl XkbState {
    fn new(keymap: &XkbKeymap) -> Option<Self> {
        let state = unsafe { (XKB.xkb_state_new)(keymap.keymap.as_ptr()) };
        NonNull::new(state).map(|state| Self { state })
    }

    #[cfg(x11_platform)]
    unsafe fn from_xcb(
        context: &XkbContext,
        keymap: &XkbKeymap,
        xcb: *mut ffi::c_void,
    ) -> Option<Self> {
        let state = (XKBX11.xkb_x11_state_new_from_device)(
            keymap.keymap.as_ptr(),
            xcb,
            context.core_keyboard,
        );

        let state = NonNull::new(state)?;
        Some(Self { state })
    }

    pub fn layout(&self) -> u32 {
        unsafe {
            (XKB.xkb_state_serialize_layout)(
                self.state.as_ptr(),
                xkb_state_component::XKB_STATE_LAYOUT_EFFECTIVE,
            )
        }
    }

    pub fn get_one_sym(&self, keycode: u32) -> Keysym {
        unsafe {
            Keysym::new((XKB.xkb_state_key_get_one_sym)(
                self.state.as_ptr(),
                keycode,
            ))
        }
    }

    pub fn update_modifiers(
        &self,
        mods_depressed: u32,
        mods_latched: u32,
        mods_locked: u32,
        depressed_group: u32,
        latched_group: u32,
        locked_group: u32,
    ) {
        unsafe {
            (XKB.xkb_state_update_mask)(
                self.state.as_ptr(),
                mods_depressed,
                mods_latched,
                mods_locked,
                depressed_group,
                latched_group,
                locked_group,
            );
        }
    }

    fn mod_name_is_active(&self, name: &[u8]) -> bool {
        unsafe {
            (XKB.xkb_state_mod_name_is_active)(
                self.state.as_ptr(),
                name.as_ptr() as *const _,
                xkb_state_component::XKB_STATE_MODS_EFFECTIVE,
            ) > 0
        }
    }

    pub fn modifiers(&self) -> Modifiers {
        Modifiers {
            shift: self.mod_name_is_active(XKB_MOD_NAME_SHIFT),
            ctrl: self.mod_name_is_active(XKB_MOD_NAME_CTRL),
            alt: self.mod_name_is_active(XKB_MOD_NAME_ALT),
            meta: self.mod_name_is_active(XKB_MOD_NAME_LOGO),
        }
    }
}

impl Drop for XkbState {
    fn drop(&mut self) {
        unsafe { (XKB.xkb_state_unref)(self.state.as_ptr()) }
    }
}

#[derive(Clone)]
pub struct XkbContext {
    inner: Rc<XkbContextInner>,
    core_keyboard: i32,
}

impl XkbContext {
    pub fn new() -> Option<Self> {
        let context = unsafe { (XKB.xkb_context_new)(XKB_CONTEXT_NO_FLAGS) };
        assert!(!context.is_null());

        let inner = XkbContextInner {
            context: NonNull::new(context)?,
        };

        Some(Self {
            inner: Rc::new(inner),
            core_keyboard: 0,
        })
    }

    #[cfg(x11_platform)]
    pub unsafe fn from_xcb(xcb: *mut ffi::c_void) -> Option<Self> {
        use xkbcommon_dl::x11::xkb_x11_setup_xkb_extension_flags::XKB_X11_SETUP_XKB_EXTENSION_NO_FLAGS;

        let result = (XKBX11.xkb_x11_setup_xkb_extension)(
            xcb,
            1,
            2,
            XKB_X11_SETUP_XKB_EXTENSION_NO_FLAGS,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );

        if result != 1 {
            return None;
        }

        let mut context = Self::new()?;
        context.core_keyboard = (XKBX11.xkb_x11_get_core_keyboard_device_id)(xcb);
        Some(context)
    }

    pub fn ptr(&self) -> *mut xkb_context {
        self.inner.context.as_ptr()
    }
}

struct XkbContextInner {
    context: NonNull<xkb_context>,
}

impl Drop for XkbContextInner {
    fn drop(&mut self) {
        unsafe { (XKB.xkb_context_unref)(self.context.as_ptr()) }
    }
}

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
