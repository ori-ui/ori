use std::{
    ffi,
    sync::{Arc, LazyLock},
};

use libloading::Library;

static LIB_EGL: LazyLock<Library> = LazyLock::new(|| {
    // load libEGL.so
    unsafe { Library::new("libEGL.so").unwrap() }
});

#[cfg(x11_platform)]
static LIB_X11: LazyLock<Library> = LazyLock::new(|| {
    // load libX11.so
    unsafe { Library::new("libX11.so").unwrap() }
});

#[allow(unused)]
struct EglContextInner {
    #[cfg(x11_platform)]
    xdisplay: *mut ffi::c_void,
    display: *mut ffi::c_void,
    config: *mut ffi::c_void,
    context: *mut ffi::c_void,
}

unsafe impl Send for EglContextInner {}
unsafe impl Sync for EglContextInner {}

pub struct EglContext {
    inner: Arc<EglContextInner>,
}

impl EglContext {
    pub fn new() -> Result<Self, EglError> {
        #[cfg(x11_platform)]
        let xdisplay = unsafe { x_open_display(std::ptr::null()) };
        let display = unsafe { egl_get_display(xdisplay) };

        let mut major = 0;
        let mut minor = 0;

        unsafe {
            egl_initialize(display, &mut major, &mut minor);
            check_egl_error()?;
        };

        unsafe {
            egl_bind_api(EGL_OPENGL_API);
            check_egl_error()?;
        }

        let config_attribs = [
            EGL_SURFACE_TYPE,
            EGL_WINDOW_BIT,
            EGL_CONFORMANT,
            EGL_OPENGL_BIT,
            EGL_RED_SIZE,
            8,
            EGL_GREEN_SIZE,
            8,
            EGL_BLUE_SIZE,
            8,
            EGL_ALPHA_SIZE,
            8,
            EGL_STENCIL_SIZE,
            8,
            EGL_NONE,
        ];

        let mut config = std::ptr::null_mut();
        let mut num_config = 0;

        unsafe {
            egl_choose_config(
                display,
                config_attribs.as_ptr(),
                &mut config,
                &mut num_config,
            );
            check_egl_error()?;
        }

        if num_config != 1 {
            panic!("no EGL config found");
        }

        let context_attribs = [
            EGL_CONTEXT_MAJOR_VERSION,
            3,
            EGL_CONTEXT_MINOR_VERSION,
            3,
            EGL_CONTEXT_OPENGL_PROFILE_MASK,
            EGL_CONTEXT_OPENGL_CORE_PROFILE_BIT,
            EGL_NONE,
        ];

        let context = unsafe {
            egl_create_context(
                display,
                config,
                std::ptr::null_mut(),
                context_attribs.as_ptr(),
            )
        };
        check_egl_error()?;

        let inner = Arc::new(EglContextInner {
            xdisplay,
            display,
            config,
            context,
        });

        Ok(Self { inner })
    }
}

impl Drop for EglContextInner {
    fn drop(&mut self) {
        unsafe {
            egl_make_current(
                self.display,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            egl_destroy_context(self.display, self.context);
            egl_terminate(self.display);
            x_close_display(self.xdisplay);
        }
    }
}

pub struct EglSurface {
    cx: Arc<EglContextInner>,
    surface: *mut ffi::c_void,
}

impl EglSurface {
    pub fn new(context: &EglContext, window: *mut ffi::c_void) -> Result<Self, EglError> {
        let surface_attribs = [EGL_NONE];

        let surface = unsafe {
            egl_create_window_surface(
                context.inner.display,
                context.inner.config,
                window,
                surface_attribs.as_ptr(),
            )
        };
        check_egl_error()?;

        Ok(Self {
            cx: context.inner.clone(),
            surface,
        })
    }

    pub fn swap_interval(&self, interval: i32) -> Result<(), EglError> {
        unsafe {
            egl_swap_interval(self.cx.display, interval);
            check_egl_error()?;
        }

        Ok(())
    }

    pub fn make_current(&self) -> Result<(), EglError> {
        unsafe {
            egl_make_current(self.cx.display, self.surface, self.surface, self.cx.context);
            check_egl_error()?;
        }

        Ok(())
    }

    pub fn swap_buffers(&self) -> Result<(), EglError> {
        unsafe {
            egl_swap_buffers(self.cx.display, self.surface);
            check_egl_error()?;
        }

        Ok(())
    }
}

impl Drop for EglSurface {
    fn drop(&mut self) {
        unsafe {
            egl_destroy_surface(self.cx.display, self.surface);
        }
    }
}

const EGL_NONE: i32 = 0x3038;

const EGL_OPENGL_API: i32 = 0x30A2;

const EGL_SURFACE_TYPE: i32 = 0x3033;
const EGL_WINDOW_BIT: i32 = 0x0004;
const EGL_CONFORMANT: i32 = 0x3042;
const EGL_OPENGL_BIT: i32 = 0x0008;
const EGL_RED_SIZE: i32 = 0x3024;
const EGL_GREEN_SIZE: i32 = 0x3023;
const EGL_BLUE_SIZE: i32 = 0x3022;
const EGL_ALPHA_SIZE: i32 = 0x3021;
const EGL_STENCIL_SIZE: i32 = 0x3026;

const EGL_CONTEXT_MAJOR_VERSION: i32 = 0x3098;
const EGL_CONTEXT_MINOR_VERSION: i32 = 0x30FB;
const EGL_CONTEXT_OPENGL_PROFILE_MASK: i32 = 0x30FD;
const EGL_CONTEXT_OPENGL_CORE_PROFILE_BIT: i32 = 0x00000001;

const EGL_SUCCESS: i32 = 0x3000;
const EGL_NOT_INITIALIZED: i32 = 0x3001;
const EGL_BAD_ACCESS: i32 = 0x3002;
const EGL_BAD_ALLOC: i32 = 0x3003;
const EGL_BAD_ATTRIBUTE: i32 = 0x3004;
const EGL_BAD_CONTEXT: i32 = 0x3005;
const EGL_BAD_CONFIG: i32 = 0x3006;
const EGL_BAD_CURRENT_SURFACE: i32 = 0x3007;
const EGL_BAD_DISPLAY: i32 = 0x3008;
const EGL_BAD_SURFACE: i32 = 0x3009;
const EGL_BAD_MATCH: i32 = 0x300A;
const EGL_BAD_PARAMETER: i32 = 0x300B;
const EGL_BAD_NATIVE_PIXMAP: i32 = 0x300C;
const EGL_BAD_NATIVE_WINDOW: i32 = 0x300D;
const EGL_CONTEXT_LOST: i32 = 0x300E;

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EglError {
    NotInitialized = EGL_NOT_INITIALIZED,
    BadAccess = EGL_BAD_ACCESS,
    BadAlloc = EGL_BAD_ALLOC,
    BadAttribute = EGL_BAD_ATTRIBUTE,
    BadContext = EGL_BAD_CONTEXT,
    BadConfig = EGL_BAD_CONFIG,
    BadCurrentSurface = EGL_BAD_CURRENT_SURFACE,
    BadDisplay = EGL_BAD_DISPLAY,
    BadSurface = EGL_BAD_SURFACE,
    BadMatch = EGL_BAD_MATCH,
    BadParameter = EGL_BAD_PARAMETER,
    BadNativePixmap = EGL_BAD_NATIVE_PIXMAP,
    BadNativeWindow = EGL_BAD_NATIVE_WINDOW,
    ContextLost = EGL_CONTEXT_LOST,
}

impl std::fmt::Display for EglError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EglError::NotInitialized => write!(f, "EGL is not initialized"),
            EglError::BadAccess => write!(f, "EGL bad access"),
            EglError::BadAlloc => write!(f, "EGL bad alloc"),
            EglError::BadAttribute => write!(f, "EGL bad attribute"),
            EglError::BadContext => write!(f, "EGL bad context"),
            EglError::BadConfig => write!(f, "EGL bad config"),
            EglError::BadCurrentSurface => write!(f, "EGL bad current surface"),
            EglError::BadDisplay => write!(f, "EGL bad display"),
            EglError::BadSurface => write!(f, "EGL bad surface"),
            EglError::BadMatch => write!(f, "EGL bad match"),
            EglError::BadParameter => write!(f, "EGL bad parameter"),
            EglError::BadNativePixmap => write!(f, "EGL bad native pixmap"),
            EglError::BadNativeWindow => write!(f, "EGL bad native window"),
            EglError::ContextLost => write!(f, "EGL context lost"),
        }
    }
}

unsafe fn x_open_display(name: *const ffi::c_char) -> *mut ffi::c_void {
    let x_open_display: libloading::Symbol<
        unsafe extern "C" fn(*const ffi::c_char) -> *mut ffi::c_void,
    > = LIB_X11.get(b"XOpenDisplay").unwrap();
    x_open_display(name)
}

unsafe fn x_close_display(display: *mut ffi::c_void) {
    let x_close_display: libloading::Symbol<unsafe extern "C" fn(*mut ffi::c_void)> =
        LIB_X11.get(b"XCloseDisplay").unwrap();
    x_close_display(display);
}

unsafe fn egl_get_error() -> i32 {
    let egl_get_error: libloading::Symbol<unsafe extern "C" fn() -> i32> =
        LIB_EGL.get(b"eglGetError").unwrap();
    egl_get_error()
}

#[track_caller]
fn check_egl_error() -> Result<(), EglError> {
    let error = unsafe { egl_get_error() };

    match error {
        EGL_SUCCESS => Ok(()),
        EGL_NOT_INITIALIZED => Err(EglError::NotInitialized),
        EGL_BAD_ACCESS => Err(EglError::BadAccess),
        EGL_BAD_ALLOC => Err(EglError::BadAlloc),
        EGL_BAD_ATTRIBUTE => Err(EglError::BadAttribute),
        EGL_BAD_CONTEXT => Err(EglError::BadContext),
        EGL_BAD_CONFIG => Err(EglError::BadConfig),
        EGL_BAD_CURRENT_SURFACE => Err(EglError::BadCurrentSurface),
        EGL_BAD_DISPLAY => Err(EglError::BadDisplay),
        EGL_BAD_SURFACE => Err(EglError::BadSurface),
        EGL_BAD_MATCH => Err(EglError::BadMatch),
        EGL_BAD_PARAMETER => Err(EglError::BadParameter),
        EGL_BAD_NATIVE_PIXMAP => Err(EglError::BadNativePixmap),
        EGL_BAD_NATIVE_WINDOW => Err(EglError::BadNativeWindow),
        EGL_CONTEXT_LOST => Err(EglError::ContextLost),
        _ => unreachable!(),
    }
}

unsafe fn egl_get_display(xdisplay: *mut ffi::c_void) -> *mut ffi::c_void {
    let egl_get_display: libloading::Symbol<
        unsafe extern "C" fn(*mut ffi::c_void) -> *mut ffi::c_void,
    > = LIB_EGL.get(b"eglGetDisplay").unwrap();
    egl_get_display(xdisplay)
}

unsafe fn egl_initialize(display: *mut ffi::c_void, major: *mut i32, minor: *mut i32) -> i32 {
    let egl_initialize: libloading::Symbol<
        unsafe extern "C" fn(*mut ffi::c_void, *mut i32, *mut i32) -> i32,
    > = LIB_EGL.get(b"eglInitialize").unwrap();
    egl_initialize(display, major, minor)
}

unsafe fn egl_terminate(display: *mut ffi::c_void) -> i32 {
    let egl_terminate: libloading::Symbol<unsafe extern "C" fn(*mut ffi::c_void) -> i32> =
        LIB_EGL.get(b"eglTerminate").unwrap();
    egl_terminate(display)
}

unsafe fn egl_bind_api(api: i32) -> i32 {
    let egl_bind_api: libloading::Symbol<unsafe extern "C" fn(i32) -> i32> =
        LIB_EGL.get(b"eglBindAPI").unwrap();
    egl_bind_api(api)
}

unsafe fn egl_swap_interval(display: *mut ffi::c_void, interval: i32) -> i32 {
    let egl_swap_interval: libloading::Symbol<unsafe extern "C" fn(*mut ffi::c_void, i32) -> i32> =
        LIB_EGL.get(b"eglSwapInterval").unwrap();
    egl_swap_interval(display, interval)
}

unsafe fn egl_choose_config(
    display: *mut ffi::c_void,
    attribs: *const i32,
    config: *mut *mut ffi::c_void,
    num_config: *mut i32,
) -> i32 {
    let egl_choose_config: libloading::Symbol<
        unsafe extern "C" fn(
            *mut ffi::c_void,
            *const i32,
            *mut *mut ffi::c_void,
            i32,
            *mut i32,
        ) -> i32,
    > = LIB_EGL.get(b"eglChooseConfig").unwrap();
    egl_choose_config(display, attribs, config, 1, num_config)
}

unsafe fn egl_create_context(
    display: *mut ffi::c_void,
    config: *mut ffi::c_void,
    share: *mut ffi::c_void,
    attribs: *const i32,
) -> *mut ffi::c_void {
    let egl_create_context: libloading::Symbol<
        unsafe extern "C" fn(
            *mut ffi::c_void,
            *mut ffi::c_void,
            *mut ffi::c_void,
            *const i32,
        ) -> *mut ffi::c_void,
    > = LIB_EGL.get(b"eglCreateContext").unwrap();
    egl_create_context(display, config, share, attribs)
}

unsafe fn egl_destroy_context(display: *mut ffi::c_void, context: *mut ffi::c_void) -> i32 {
    let egl_destroy_context: libloading::Symbol<
        unsafe extern "C" fn(*mut ffi::c_void, *mut ffi::c_void) -> i32,
    > = LIB_EGL.get(b"eglDestroyContext").unwrap();
    egl_destroy_context(display, context)
}

unsafe fn egl_create_window_surface(
    display: *mut ffi::c_void,
    config: *mut ffi::c_void,
    window: *mut ffi::c_void,
    attribs: *const i32,
) -> *mut ffi::c_void {
    let egl_create_window_surface: libloading::Symbol<
        unsafe extern "C" fn(
            *mut ffi::c_void,
            *mut ffi::c_void,
            *mut ffi::c_void,
            *const i32,
        ) -> *mut ffi::c_void,
    > = LIB_EGL.get(b"eglCreateWindowSurface").unwrap();
    egl_create_window_surface(display, config, window, attribs)
}

unsafe fn egl_destroy_surface(display: *mut ffi::c_void, surface: *mut ffi::c_void) -> i32 {
    let egl_destroy_surface: libloading::Symbol<
        unsafe extern "C" fn(*mut ffi::c_void, *mut ffi::c_void) -> i32,
    > = LIB_EGL.get(b"eglDestroySurface").unwrap();
    egl_destroy_surface(display, surface)
}

unsafe fn egl_make_current(
    display: *mut ffi::c_void,
    draw: *mut ffi::c_void,
    read: *mut ffi::c_void,
    context: *mut ffi::c_void,
) -> i32 {
    let egl_make_current: libloading::Symbol<
        unsafe extern "C" fn(
            *mut ffi::c_void,
            *mut ffi::c_void,
            *mut ffi::c_void,
            *mut ffi::c_void,
        ) -> i32,
    > = LIB_EGL.get(b"eglMakeCurrent").unwrap();
    egl_make_current(display, draw, read, context)
}

unsafe fn egl_swap_buffers(display: *mut ffi::c_void, surface: *mut ffi::c_void) -> i32 {
    let egl_swap_buffers: libloading::Symbol<
        unsafe extern "C" fn(*mut ffi::c_void, *mut ffi::c_void) -> i32,
    > = LIB_EGL.get(b"eglSwapBuffers").unwrap();
    egl_swap_buffers(display, surface)
}
