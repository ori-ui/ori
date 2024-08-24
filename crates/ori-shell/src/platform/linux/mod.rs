#[allow(unused)]
pub mod egl;
#[allow(unused)]
pub mod xkb;

use std::sync::LazyLock;

use libloading::Library;

#[allow(unused)]
pub static LIB_GL: LazyLock<Library> = LazyLock::new(|| {
    // load libGL.so
    unsafe { Library::new("libGL.so").unwrap() }
});
