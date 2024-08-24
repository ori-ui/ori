use std::{
    fs::File,
    os::fd::OwnedFd,
    ptr::{self, NonNull},
    rc::Rc,
    sync::LazyLock,
};

use memmap::MmapOptions;
use ori_core::event::{Key, Modifiers};
use xkbcommon_dl::{
    xkb_context, xkb_context_flags::XKB_CONTEXT_NO_FLAGS, xkb_keymap,
    xkb_keymap_compile_flags::XKB_KEYMAP_COMPILE_NO_FLAGS,
    xkb_keymap_format::XKB_KEYMAP_FORMAT_TEXT_V1, xkb_state, xkb_state_component, xkbcommon_handle,
    XkbCommon, XKB_MOD_NAME_ALT, XKB_MOD_NAME_CTRL, XKB_MOD_NAME_LOGO, XKB_MOD_NAME_SHIFT,
};
use xkeysym::Keysym;

static XKB: LazyLock<&'static XkbCommon> = LazyLock::new(xkbcommon_handle);

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
    fn from_fd(context: &XkbContext, fd: OwnedFd, size: usize) -> Option<Self> {
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
        })
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
