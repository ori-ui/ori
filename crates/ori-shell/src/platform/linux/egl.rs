use std::{
    ffi, ptr,
    rc::Rc,
    sync::{Arc, LazyLock},
};

use libloading::Library;

static LIB_EGL: LazyLock<Result<Library, Arc<libloading::Error>>> = LazyLock::new(|| {
    // load libEGL.so
    unsafe { Library::new("libEGL.so").map_err(Arc::new) }
});

#[derive(Debug)]
#[non_exhaustive]
pub enum EglNativeDisplay {
    #[cfg(x11_platform)]
    X11,

    #[cfg(wayland_platform)]
    Wayland(*mut ffi::c_void),
}

impl EglNativeDisplay {
    fn egl_platform(&self) -> i32 {
        match self {
            #[cfg(x11_platform)]
            Self::X11 => EGL_PLATFORM_X11,

            #[cfg(wayland_platform)]
            Self::Wayland(_) => EGL_PLATFORM_WAYLAND,

            _ => unreachable!(),
        }
    }

    fn as_ptr(&self) -> *mut ffi::c_void {
        match self {
            #[cfg(x11_platform)]
            Self::X11 => ptr::null_mut(),

            #[cfg(wayland_platform)]
            Self::Wayland(ptr) => *ptr,

            _ => unreachable!(),
        }
    }
}

#[allow(unused)]
struct EglContextInner {
    native: EglNativeDisplay,
    display: *mut ffi::c_void,
    config: *mut ffi::c_void,
    context: *mut ffi::c_void,
}

pub struct EglContext {
    inner: Rc<EglContextInner>,
}

impl EglContext {
    pub fn new(native_display: EglNativeDisplay) -> Result<Self, EglError> {
        let display = unsafe {
            egl_get_platform_display(native_display.egl_platform(), native_display.as_ptr())?
        };

        let mut major = 0;
        let mut minor = 0;

        unsafe {
            egl_initialize(display, &mut major, &mut minor)?;
        };

        unsafe {
            egl_bind_api(EGL_OPENGL_API)?;
        }

        let config_attribs = [
            EGL_SURFACE_TYPE,
            EGL_WINDOW_BIT,
            EGL_CONFORMANT,
            EGL_OPENGL_BIT,
            EGL_RENDERABLE_TYPE,
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

        let mut config = ptr::null_mut();
        let mut num_config = 0;

        unsafe {
            egl_choose_config(
                display,
                config_attribs.as_ptr(),
                &mut config,
                &mut num_config,
            )?;
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
            egl_create_context(display, config, ptr::null_mut(), context_attribs.as_ptr())?
        };

        let inner = Rc::new(EglContextInner {
            native: native_display,
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
            let _ = egl_make_current(
                self.display,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            );
            egl_destroy_context(self.display, self.context);
            egl_terminate(self.display);
        }
    }
}

pub struct EglSurface {
    cx: Rc<EglContextInner>,
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
            )?
        };

        Ok(Self {
            cx: context.inner.clone(),
            surface,
        })
    }

    pub fn from_raw_surface(context: &EglContext, surface: *mut ffi::c_void) -> Self {
        Self {
            cx: context.inner.clone(),
            surface,
        }
    }

    pub fn swap_interval(&self, interval: i32) -> Result<(), EglError> {
        unsafe {
            egl_swap_interval(self.cx.display, interval);
        }

        Ok(())
    }

    pub fn make_current(&self) -> Result<(), EglError> {
        unsafe {
            egl_make_current(self.cx.display, self.surface, self.surface, self.cx.context)?;
        }

        Ok(())
    }

    pub fn swap_buffers(&self) -> Result<(), EglError> {
        unsafe {
            egl_swap_buffers(self.cx.display, self.surface)?;
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
const EGL_RENDERABLE_TYPE: i32 = 0x3040;
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

#[cfg(x11_platform)]
const EGL_PLATFORM_X11: i32 = 0x31D5;

#[cfg(wayland_platform)]
const EGL_PLATFORM_WAYLAND: i32 = 0x31D8;

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

#[derive(Debug)]
pub enum EglError {
    Library(Arc<libloading::Error>),
    NotInitialized,
    BadAccess,
    BadAlloc,
    BadAttribute,
    BadContext,
    BadConfig,
    BadCurrentSurface,
    BadDisplay,
    BadSurface,
    BadMatch,
    BadParameter,
    BadNativePixmap,
    BadNativeWindow,
    ContextLost,
}

impl From<libloading::Error> for EglError {
    fn from(err: libloading::Error) -> Self {
        Self::Library(Arc::new(err))
    }
}

impl From<&Arc<libloading::Error>> for EglError {
    fn from(err: &Arc<libloading::Error>) -> Self {
        Self::Library(err.clone())
    }
}

impl std::fmt::Display for EglError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EglError::Library(err) => write!(f, "EGL library error: {}", err),
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

fn lib_egl() -> Result<&'static Library, EglError> {
    Ok(LIB_EGL.as_ref()?)
}

unsafe fn lib_egl_symbol<T>(name: &[u8]) -> Result<libloading::Symbol<T>, EglError> {
    Ok(lib_egl()?.get(name)?)
}

unsafe fn egl_get_error() -> Result<i32, EglError> {
    let egl_get_error: libloading::Symbol<unsafe extern "C" fn() -> i32> =
        lib_egl_symbol(b"eglGetError")?;

    Ok(egl_get_error())
}

#[track_caller]
fn check_egl_error() -> Result<(), EglError> {
    let error = unsafe { egl_get_error()? };

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

unsafe fn egl_get_platform_display(
    platform: i32,
    native_display: *mut ffi::c_void,
) -> Result<*mut ffi::c_void, EglError> {
    let egl_get_platform_display: libloading::Symbol<
        unsafe extern "C" fn(i32, *mut ffi::c_void, *const i32) -> *mut ffi::c_void,
    > = lib_egl_symbol(b"eglGetPlatformDisplay")?;

    Ok(egl_get_platform_display(
        platform,
        native_display,
        ptr::null(),
    ))
}

unsafe fn egl_initialize(
    display: *mut ffi::c_void,
    major: *mut i32,
    minor: *mut i32,
) -> Result<(), EglError> {
    let egl_initialize: libloading::Symbol<
        unsafe extern "C" fn(*mut ffi::c_void, *mut i32, *mut i32) -> i32,
    > = lib_egl_symbol(b"eglInitialize")?;

    let result = egl_initialize(display, major, minor);

    if result == 0 {
        check_egl_error()?;
    }

    Ok(())
}

unsafe fn egl_terminate(display: *mut ffi::c_void) {
    let egl_terminate =
        lib_egl_symbol::<unsafe extern "C" fn(*mut ffi::c_void) -> i32>(b"eglTerminate");

    if let Ok(egl_terminate) = egl_terminate {
        egl_terminate(display);
    }
}

unsafe fn egl_bind_api(api: i32) -> Result<(), EglError> {
    let egl_bind_api = lib_egl_symbol::<unsafe extern "C" fn(i32) -> i32>(b"eglBindAPI")?;

    egl_bind_api(api);

    Ok(())
}

unsafe fn egl_choose_config(
    display: *mut ffi::c_void,
    attrib_list: *const i32,
    config: *mut *mut ffi::c_void,
    num_config: *mut i32,
) -> Result<(), EglError> {
    let egl_choose_config: libloading::Symbol<
        unsafe extern "C" fn(
            *mut ffi::c_void,
            *const i32,
            *mut *mut ffi::c_void,
            i32,
            *mut i32,
        ) -> i32,
    > = lib_egl_symbol(b"eglChooseConfig")?;

    let result = egl_choose_config(display, attrib_list, config, 1, num_config);

    if result == 0 {
        check_egl_error()?;
    }

    Ok(())
}

unsafe fn egl_create_context(
    display: *mut ffi::c_void,
    config: *mut ffi::c_void,
    share_context: *mut ffi::c_void,
    attrib_list: *const i32,
) -> Result<*mut ffi::c_void, EglError> {
    let egl_create_context: libloading::Symbol<
        unsafe extern "C" fn(
            *mut ffi::c_void,
            *mut ffi::c_void,
            *mut ffi::c_void,
            *const i32,
        ) -> *mut ffi::c_void,
    > = lib_egl_symbol(b"eglCreateContext")?;

    let context = egl_create_context(display, config, share_context, attrib_list);

    if context.is_null() {
        check_egl_error()?;
    }

    Ok(context)
}

unsafe fn egl_destroy_context(display: *mut ffi::c_void, context: *mut ffi::c_void) {
    let egl_destroy_context = lib_egl_symbol::<
        unsafe extern "C" fn(*mut ffi::c_void, *mut ffi::c_void),
    >(b"eglDestroyContext");

    if let Ok(egl_destroy_context) = egl_destroy_context {
        egl_destroy_context(display, context);
    }
}

unsafe fn egl_create_window_surface(
    display: *mut ffi::c_void,
    config: *mut ffi::c_void,
    native_window: *mut ffi::c_void,
    attrib_list: *const i32,
) -> Result<*mut ffi::c_void, EglError> {
    let egl_create_window_surface: libloading::Symbol<
        unsafe extern "C" fn(
            *mut ffi::c_void,
            *mut ffi::c_void,
            *mut ffi::c_void,
            *const i32,
        ) -> *mut ffi::c_void,
    > = lib_egl_symbol(b"eglCreateWindowSurface")?;

    let surface = egl_create_window_surface(display, config, native_window, attrib_list);

    if surface.is_null() {
        check_egl_error()?;
    }

    Ok(surface)
}

unsafe fn egl_destroy_surface(display: *mut ffi::c_void, surface: *mut ffi::c_void) {
    let egl_destroy_surface = lib_egl_symbol::<
        unsafe extern "C" fn(*mut ffi::c_void, *mut ffi::c_void),
    >(b"eglDestroySurface");

    if let Ok(egl_destroy_surface) = egl_destroy_surface {
        egl_destroy_surface(display, surface);
    }
}

unsafe fn egl_make_current(
    display: *mut ffi::c_void,
    draw: *mut ffi::c_void,
    read: *mut ffi::c_void,
    context: *mut ffi::c_void,
) -> Result<(), EglError> {
    let egl_make_current: libloading::Symbol<
        unsafe extern "C" fn(
            *mut ffi::c_void,
            *mut ffi::c_void,
            *mut ffi::c_void,
            *mut ffi::c_void,
        ) -> i32,
    > = lib_egl_symbol(b"eglMakeCurrent")?;

    let result = egl_make_current(display, draw, read, context);

    if result == 0 {
        check_egl_error()?;
    }

    Ok(())
}

unsafe fn egl_swap_interval(display: *mut ffi::c_void, interval: i32) {
    let egl_swap_interval =
        lib_egl_symbol::<unsafe extern "C" fn(*mut ffi::c_void, i32)>(b"eglSwapInterval");

    if let Ok(egl_swap_interval) = egl_swap_interval {
        egl_swap_interval(display, interval);
    }
}

unsafe fn egl_swap_buffers(
    display: *mut ffi::c_void,
    surface: *mut ffi::c_void,
) -> Result<(), EglError> {
    let egl_swap_buffers: libloading::Symbol<
        unsafe extern "C" fn(*mut ffi::c_void, *mut ffi::c_void) -> i32,
    > = lib_egl_symbol(b"eglSwapBuffers")?;

    let result = egl_swap_buffers(display, surface);

    if result == 0 {
        check_egl_error()?;
    }

    Ok(())
}
