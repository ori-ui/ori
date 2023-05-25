//! Ori is a framework for building cross-platform native gui applications.
//!
//! Ori uses a reactive, declarative programming model to build applications.
//! This means that the application is built by composing components together.
//!
//! # Example
//! ```no_run
//! use ori::prelude::*;
//!
//! fn ui(cx: Scope) -> impl View {
//!     let counter = cx.signal(0);
//!
//!     view! {
//!         <Button on:press=move |_| *counter.modify() += 1>
//!             "Click me!"
//!         </Button>
//!         { format!("Clicked {} times", counter.get()) }
//!     }
//! }
//!
//! fn main() {
//!    App::new(ui).run();
//! }
//! ```

pub use ori_core as core;
pub use ori_graphics as graphics;
pub use ori_reactive as reactive;

#[cfg(feature = "baseview")]
pub use ori_baseview as baseview;
#[cfg(feature = "winit")]
pub use ori_winit as winit;

#[cfg(feature = "ash")]
pub use ori_ash as ash;
#[cfg(feature = "wgpu")]
pub use ori_wgpu as wgpu;

pub mod prelude {
    pub use ori_core::prelude::*;
    pub use ori_graphics::prelude::*;
    pub use ori_reactive::prelude::*;

    #[cfg(feature = "winit")]
    pub use ori_winit::App;

    #[cfg(feature = "baseview")]
    pub use ori_baseview::App;
}
