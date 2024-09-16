#![deny(missing_docs)]

//! Forces ori to be built as a dynamic library, which considerably reduces the
//! compile times when iterating on the code.
//!
//! # Warning
//! __Do not use this in production!__ This is only meant to be used during development.
//! If used in production, would require you to ship the dylib with the binary.

#[cfg_attr(not(feature = "always"), cfg(debug_assertions))]
#[allow(unused_imports, clippy::single_component_path_imports)]
mod dylib {
    use ori_core;

    #[cfg(feature = "shell")]
    use ori_shell;
}
