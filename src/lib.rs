#![warn(missing_docs)]

//! Ori is a cross-platform declarative UI framework for Rust, with a focus on
//! simplicity and performance.

pub mod core {
    //! Ori [`core`](ori_core) module.

    pub use ori_core::*;
}

#[cfg(feature = "font-awesome")]
pub mod font_awesome {
    //! Ori [`font-awesome`](ori_font_awesome) integration.

    pub use ori_font_awesome::*;
}

#[cfg(feature = "wgpu")]
pub mod wgpu {
    //! Ori [`wgpu`](ori_wgpu) integration.

    pub use ori_wgpu::*;
}

pub mod prelude {
    //! Convenient imports for Ori.

    #[cfg(feature = "wgpu")]
    pub use crate::wgpu::App;

    pub use ori_core::{
        canvas::{
            hex, hsl, hsla, rgb, rgba, BorderRadius, BorderWidth, Canvas, Color, Curve, Fragment,
            Mesh, Primitive, Vertex,
        },
        delegate::{Delegate, DelegateCx},
        event::{
            Code, Event, KeyboardEvent, Modifiers, Pointer, PointerButton, PointerEvent, PointerId,
        },
        image::{Image, ImageData, ImageId},
        layout::{Affine, Align, Alignment, Axis, Justify, Padding, Rect, Size, Space},
        log::*,
        math::*,
        rebuild::Rebuild,
        text::{
            font, FontFamily, FontSource, FontStretch, FontStyle, FontWeight, Fonts, FontsError,
            Glyph, Glyphs, TextAlign, TextSection, TextWrap,
        },
        theme::{builtin::*, em, pt, set_style, set_theme, style, styled, Key, Palette, Theme},
        transition::{ease, linear, Transition, TransitionCurve},
        view::{
            any, AnyView, BoxedView, BuildCx, Content, ContentSeq, DrawCx, EventCx, LayoutCx,
            RebuildCx, SeqState, State, View, ViewSeq, ViewState,
        },
        views::*,
        window::{Cursor, Window, WindowDescriptor, WindowId},
    };

    #[cfg(feature = "font-awesome")]
    pub use ori_font_awesome as fa;

    #[cfg(feature = "image")]
    pub use ori_core::image;
}
