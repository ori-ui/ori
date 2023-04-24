pub use ori_core as core;
pub use ori_graphics as graphics;

pub mod prelude {
    pub use ori_core::*;
    pub use ori_graphics::*;
    pub use ori_macro::*;

    #[cfg(feature = "winit")]
    pub use ori_winit::App;
}
