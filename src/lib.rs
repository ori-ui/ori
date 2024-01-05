#![warn(missing_docs)]

//! Ori is a cross-platform declarative UI framework for Rust, with a focus on
//! simplicity and performance.
//!
//! Ori is heavily inspired by SwiftUI and xilem, and uses a similar approach
//! to building user interfaces. It is built on top of [`ori_core`], which
//! provides the core functionality, and [`ori_winit`], which provides
//! a winit based shell, and supports both a wgpu, and glow based renderer.
//!
//! # Examples
//! For more examples, see [`ori/examples`](https://github.com/ChangeCaps/ori/tree/main/examples).
//!
//! ```rust,no_run
//! use ori::prelude::*;
//!
//! // define the data model. this can be anything that implements 'static.
//! //
//! // here we just have a struct with a single field.
//! #[derive(Default)]
//! struct Data {
//!     count: u32,
//! }
//!
//! // define the user interface builder. this is a function that takes a
//! // mutable reference to the Data and returns a View of the data.
//! //
//! // this function is called once when the window is created, and again
//! // whenever a rebuild is requested. it is therefore important that this
//! // function is cheap to call, as it might be called many times per second.
//! fn ui(data: &mut Data) -> impl View<Data> {
//!     // create a Text view with the current count. all builtin views have
//!     // shorthand functions for creating them, note that Text::new() is
//!     // would work just as well.
//!     let count = text(format!("Clicked {} times", data.count));
//!
//!     // create a button that increments the count when clicked.
//!     let count = on_click(
//!         // we use the button view to add some visual feedback.
//!         button(count).fancy(4.0),
//!         // this is the event handler, it is called when the content is clicked.
//!         // note that this implicitly requests a rebuild of the view tree.
//!         |_, data: &mut Data| data.count += 1,
//!     );
//!
//!     // finally we center the the button in the window.
//!     //
//!     // it should be noted that () implements View, a common issue is to
//!     // accidentally add a semicolon after the last line, which will cause
//!     // the function to return (), which can cause some confusing errors.
//!     center(count)
//! }
//!
//! fn main() {
//!     // create a window descriptor.
//!     let window = WindowDescriptor::new()
//!         .title("Hello, world!"); // set the title of the window.
//!
//!     // create a launcher with the data model.
//!     Launcher::new(Data::default())
//!         .window(window, ui) // add the window with the ui builder.
//!         .launch(); // launch the application.
//! }
//! ```

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

pub mod prelude {
    //! Convenient imports for Ori.

    /// Type alias for [`ori_core::launcher::Launcher`] with [`ori_winit::WinitShell`].
    #[cfg(feature = "winit")]
    pub type Launcher<T> = ori_core::launcher::Launcher<T, ori_winit::WinitShell>;

    pub use ori_core::{
        canvas::{
            hex, hsl, hsla, oklab, oklaba, rgb, rgba, BorderRadius, BorderWidth, BoxShadow, Canvas,
            Color, Curve, Fragment, Mesh, Primitive, Vertex,
        },
        delegate::{Delegate, DelegateCx},
        event::{
            ActiveChanged, AnimationFrame, CloseRequested, CloseWindow, Code, Event, HotChanged,
            KeyboardEvent, Modifiers, OpenWindow, Pointer, PointerButton, PointerId, PointerLeft,
            PointerMoved, PointerPressed, PointerReleased, PointerScrolled, RequestFocus,
        },
        image::{gradient, Image, ImageData, ImageId},
        layout::{
            Affine, Align, Alignment, Axis, Justify, Matrix, Padding, Point, Rect, Size, Space,
            Vector, FILL,
        },
        log::*,
        rebuild::Rebuild,
        text::{
            font, FontFamily, FontSource, FontStretch, FontStyle, FontWeight, Fonts, FontsError,
            Glyph, Glyphs, TextAlign, TextSection, TextWrap,
        },
        theme::{
            builtin::*, scale_factor, set_style, set_theme, style, theme_snapshot, vh, vw,
            window_size, Key, Palette, Theme,
        },
        transition::{ease, linear, Transition, TransitionCurve},
        view::{
            any, pod, AnyView, BoxedView, BuildCx, DrawCx, EventCx, LayoutCx, Pod, PodSeq,
            RebuildCx, SeqState, State, View, ViewSeq, ViewState,
        },
        views::*,
        window::{Cursor, Window, WindowDescriptor, WindowId},
    };

    pub use ori_macro::Build;

    #[cfg(feature = "font-awesome")]
    pub use ori_font_awesome as fa;

    #[cfg(feature = "image")]
    pub use ori_core::image;
}
