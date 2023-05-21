pub use ori_core as core;
pub use ori_graphics as graphics;
pub use ori_reactive as reactive;
#[cfg(feature = "wgpu")]
pub use ori_wgpu as wgpu;
#[cfg(feature = "winit")]
pub use ori_winit as winit;

pub mod prelude {
    pub use ori_core::*;
    pub use ori_graphics::*;
    pub use ori_macro::{view, Build};
    pub use ori_reactive::prelude::*;

    #[cfg(feature = "winit")]
    pub use ori_winit::App;
}
