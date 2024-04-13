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

pub mod prelude {
    //! Convenient imports for Ori.

    pub use ori_app::{App, AppBuilder, Delegate, DelegateCx};

    #[allow(unused_imports)]
    pub use ori_core::{
        canvas::{
            hex, hsl, hsla, oklab, oklaba, rgb, rgba, Background, BorderRadius, BorderWidth,
            BoxShadow, Canvas, Color, Curve, Fragment, Mesh, Primitive, Vertex,
        },
        clipboard::Clipboard,
        command::CommandProxy,
        context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
        event::{
            AnimationFrame, CloseRequested, Code, Event, KeyPressed, Modifiers, Pointer,
            PointerButton, PointerId, PointerLeft, PointerMoved, PointerPressed, PointerReleased,
            PointerScrolled, Quit, RequestFocus,
        },
        image::{gradient, Image, ImageData, ImageId},
        layout::{
            Affine, Align, Alignment, Axis, Justify, Matrix, Padding, Point, Rect, Size, Space,
            Vector, FILL,
        },
        log::{debug, error, info, trace, warn},
        rebuild::Rebuild,
        style::{palette, style, style_or, styled, Palette, Style, Styles},
        text::{
            include_font, FontFamily, FontSource, FontStretch, FontStyle, FontWeight, Fonts,
            TextAlign, TextBuffer, TextWrap,
        },
        transition::{ease, linear, Transition, TransitionCurve},
        view::{
            any, pod, AnyView, BoxedView, Pod, PodSeq, SeqState, State, View, ViewSeq, ViewState,
        },
        views::*,
        window::{Cursor, Window, WindowDescriptor, WindowId},
    };

    pub use ori_macro::Build;

    #[cfg(feature = "font-awesome")]
    pub use ori_font_awesome as fa;

    #[cfg(feature = "image")]
    pub use ori_core::include_image;
}
