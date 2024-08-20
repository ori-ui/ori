#[allow(unused)]
mod egl;
#[allow(unused)]
mod xkb;

#[allow(unused_imports)]
pub use egl::*;
#[allow(unused_imports)]
pub use xkb::*;

use std::sync::LazyLock;

use libloading::Library;

#[allow(unused)]
pub static LIB_GL: LazyLock<Library> = LazyLock::new(|| {
    // load libGL.so
    unsafe { Library::new("libGL.so").unwrap() }
});
