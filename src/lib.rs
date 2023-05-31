//! Ori is a framework for building cross-platform native gui applications.
//!
//! Ori uses a reactive, declarative programming model to build applications.
//! This means that the application is built by composing components together.
//!
//! # Example
//! ```no_run
//! use ori::prelude::*;
//!
//! // ui is a function that takes a Scope returns a Node
//! fn ui(cx: Scope) -> Node {
//!     // create a signal that will hold the state of the counter
//!     let counter = cx.signal(0);
//!
//!     // render the ui using the view! macro
//!     view! {
//!         <Button on:click=move |_| *counter.modify() += 1>
//!             "Click me!"
//!         </Button>
//!         { format!("Clicked {} times", counter.get()) }
//!     }
//! }
//!
//! fn main() {
//!     // start the application
//!     App::new(ui).run();
//! }
//! ```

pub use ori_core as core;
pub use ori_graphics as graphics;
pub use ori_reactive as reactive;
pub use ori_style as style;

#[cfg(feature = "winit")]
pub use ori_winit as winit;

#[cfg(feature = "wgpu")]
pub use ori_wgpu as wgpu;

pub mod prelude {
    //! A collection of commonly used types and traits.

    pub use ori_core::prelude::*;
    pub use ori_graphics::prelude::*;
    pub use ori_reactive::prelude::*;
    pub use ori_style::prelude::*;

    #[cfg(feature = "winit")]
    pub use ori_winit::prelude::*;
}
