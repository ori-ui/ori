#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub use ori_macro::main;

pub mod core {
    //! Ori [`core`](ori_core) module.

    pub use ori_core::*;
}

#[cfg(feature = "font-awesome")]
pub mod font_awesome {
    //! Ori [`font-awesome`](ori_font_awesome) integration.

    pub use ori_font_awesome::*;
}

#[cfg(feature = "winit")]
pub mod winit {
    //! Ori [`winit`](ori_winit) integration.

    pub use ori_winit::*;
}

#[cfg(feature = "winit")]
pub use ori_winit::launch;

#[cfg(feature = "shell")]
pub use ori_shell::{launch, launch_simple};

pub mod prelude {
    //! Convenient imports for Ori.

    pub use ori_app::{App, AppBuilder, AppCommand, Delegate, DelegateCx};

    pub use ori_core::prelude::*;

    pub use ori_macro::{desktop, mobile, Build};

    #[cfg(feature = "font-awesome")]
    pub use ori_font_awesome as fa;

    #[cfg(feature = "image")]
    pub use ori_core::include_image;
}
