#![warn(missing_docs)]

//! Ori is a cross-platform declarative UI framework for Rust, with a focus on
//! simplicity and performance.

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

    #[cfg(feature = "winit")]
    pub use crate::winit::App;

    pub use ori_core::{
        canvas::{
            hex, hsl, hsla, rgb, rgba, BorderRadius, BorderWidth, Canvas, Color, Curve, Fragment,
            Mesh, Primitive, Vertex,
        },
        delegate::{Delegate, DelegateCx},
        event::{
            AnimationFrame, Code, Event, Focused, HotChanged, KeyboardEvent, Modifiers, Pointer,
            PointerButton, PointerEvent, PointerId,
        },
        image::{Image, ImageData, ImageId},
        layout::{
            Affine, Align, Alignment, Axis, Justify, Matrix, Padding, Point, Rect, Size, Space,
            Vector,
        },
        log::*,
        rebuild::Rebuild,
        text::{
            font, FontFamily, FontSource, FontStretch, FontStyle, FontWeight, Fonts, FontsError,
            Glyph, Glyphs, TextAlign, TextSection, TextWrap,
        },
        theme::{builtin::*, pt, rem, set_style, set_theme, style, themed, Key, Palette, Theme},
        transition::{ease, linear, Transition, TransitionCurve},
        view::{
            any, AnyView, BoxedView, BuildCx, DrawCx, EventCx, LayoutCx, Pod, PodSeq, RebuildCx,
            SeqState, State, View, ViewSeq, ViewState,
        },
        views::*,
        window::{Cursor, Window, WindowDescriptor, WindowId},
    };

    #[cfg(feature = "font-awesome")]
    pub use ori_font_awesome as fa;

    #[cfg(feature = "image")]
    pub use ori_core::image;
}
