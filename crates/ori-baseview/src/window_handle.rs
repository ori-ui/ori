use old_raw_window_handle as old;
use raw_window_handle::{
    AppKitDisplayHandle, AppKitWindowHandle, HasRawDisplayHandle, HasRawWindowHandle,
    OrbitalDisplayHandle, OrbitalWindowHandle, RawDisplayHandle, RawWindowHandle,
    UiKitDisplayHandle, UiKitWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
    Win32WindowHandle, WinRtWindowHandle, WindowsDisplayHandle, XcbDisplayHandle, XcbWindowHandle,
    XlibDisplayHandle, XlibWindowHandle,
};

#[repr(transparent)]
pub(crate) struct OldWindowHandle<'a, T: old::HasRawWindowHandle>(pub(crate) &'a mut T);

unsafe impl<T: old::HasRawWindowHandle> HasRawDisplayHandle for OldWindowHandle<'_, T> {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        match self.0.raw_window_handle() {
            old::RawWindowHandle::UiKit(_h) => {
                let d = UiKitDisplayHandle::empty();
                RawDisplayHandle::UiKit(d)
            }
            old::RawWindowHandle::AppKit(_h) => {
                let d = AppKitDisplayHandle::empty();
                RawDisplayHandle::AppKit(d)
            }
            old::RawWindowHandle::Orbital(_h) => {
                let d = OrbitalDisplayHandle::empty();
                RawDisplayHandle::Orbital(d)
            }
            old::RawWindowHandle::Xlib(h) => {
                let mut d = XlibDisplayHandle::empty();
                d.display = h.display;
                d.screen = 0;

                RawDisplayHandle::Xlib(d)
            }
            old::RawWindowHandle::Xcb(h) => {
                let mut d = XcbDisplayHandle::empty();
                d.connection = h.connection;
                d.screen = 0;

                RawDisplayHandle::Xcb(d)
            }
            old::RawWindowHandle::Wayland(h) => {
                let mut d = WaylandDisplayHandle::empty();
                d.display = h.display;

                RawDisplayHandle::Wayland(d)
            }
            old::RawWindowHandle::Win32(_h) => {
                let d = WindowsDisplayHandle::empty();
                RawDisplayHandle::Windows(d)
            }
            old::RawWindowHandle::WinRt(_h) => {
                let d = WindowsDisplayHandle::empty();
                RawDisplayHandle::Windows(d)
            }
            _ => unimplemented!("Unsupported platform"),
        }
    }
}

unsafe impl<T: old::HasRawWindowHandle> HasRawWindowHandle for OldWindowHandle<'_, T> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        match self.0.raw_window_handle() {
            old::RawWindowHandle::UiKit(h) => {
                let mut w = UiKitWindowHandle::empty();
                w.ui_view = h.ui_view;
                w.ui_window = h.ui_window;
                w.ui_view_controller = h.ui_view_controller;

                RawWindowHandle::UiKit(w)
            }
            old::RawWindowHandle::AppKit(h) => {
                let mut w = AppKitWindowHandle::empty();
                w.ns_window = h.ns_window;
                w.ns_view = h.ns_view;

                RawWindowHandle::AppKit(w)
            }
            old::RawWindowHandle::Orbital(h) => {
                let mut w = OrbitalWindowHandle::empty();
                w.window = h.window;

                RawWindowHandle::Orbital(w)
            }
            old::RawWindowHandle::Xlib(h) => {
                let mut w = XlibWindowHandle::empty();
                w.window = h.window;
                w.visual_id = h.visual_id;

                RawWindowHandle::Xlib(w)
            }
            old::RawWindowHandle::Xcb(h) => {
                let mut w = XcbWindowHandle::empty();
                w.window = h.window;
                w.visual_id = h.visual_id;

                RawWindowHandle::Xcb(w)
            }
            old::RawWindowHandle::Wayland(h) => {
                let mut w = WaylandWindowHandle::empty();
                w.surface = h.surface;

                RawWindowHandle::Wayland(w)
            }
            old::RawWindowHandle::Win32(h) => {
                let mut w = Win32WindowHandle::empty();
                w.hwnd = h.hwnd;
                w.hinstance = h.hinstance;

                RawWindowHandle::Win32(w)
            }
            old::RawWindowHandle::WinRt(h) => {
                let mut w = WinRtWindowHandle::empty();
                w.core_window = h.core_window;

                RawWindowHandle::WinRt(w)
            }
            _ => unimplemented!("Unsupported platform"),
        }
    }
}
