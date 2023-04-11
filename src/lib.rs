pub use ily_core as core;
pub use ily_graphics as graphics;

pub mod prelude {
    pub use ily_core::*;
    pub use ily_graphics::*;
    pub use ily_macro::*;

    #[cfg(feature = "winit")]
    pub use ily_winit::App;
}
