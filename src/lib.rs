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

pub mod prelude {
    //! Convenient imports for Ori.

    /// Type alias for [`ori_core::launcher::Launcher`] with [`ori_winit::WinitShell`].
    #[cfg(feature = "winit")]
    pub type Launcher<T> = ori_core::launcher::Launcher<T, ori_winit::WinitShell>;

    #[allow(unused_imports)]
    pub use ori_core::{
        canvas::{
            hex, hsl, hsla, oklab, oklaba, rgb, rgba, Background, BorderRadius, BorderWidth,
            BoxShadow, Canvas, Color, Curve, Fragment, Mesh, Primitive, Vertex,
        },
        clipboard::Clipboard,
        command::CommandProxy,
        delegate::Delegate,
        event::{
            AnimationFrame, CloseRequested, CloseWindow, Code, Event, KeyPressed, Modifiers,
            OpenWindow, Pointer, PointerButton, PointerId, PointerLeft, PointerMoved,
            PointerPressed, PointerReleased, PointerScrolled, Quit, RequestFocus,
        },
        image::{gradient, Image, ImageData, ImageId},
        layout::{
            Affine, Align, Alignment, Axis, Justify, Matrix, Padding, Point, Rect, Size, Space,
            Vector, FILL,
        },
        log::*,
        rebuild::Rebuild,
        text,
        text::{
            font, FontFamily, FontSource, FontStretch, FontStyle, FontWeight, Fonts, TextAlign,
            TextBuffer, TextWrap,
        },
        theme::{style, style_or, Key, Palette, Theme},
        transition::{ease, linear, Transition, TransitionCurve},
        view::{
            any, pod, AnyView, BoxedView, BuildCx, DelegateCx, DrawCx, EventCx, LayoutCx, Pod,
            PodSeq, RebuildCx, SeqState, State, View, ViewSeq, ViewState,
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
