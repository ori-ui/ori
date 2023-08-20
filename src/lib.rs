#![warn(missing_docs)]

//! Ori is a cross-platform declarative UI framework for Rust, with a focus on
//! simplicity and performance.

pub use ori_core as core;
pub use ori_wgpu as wgpu;

pub mod prelude {
    //! The `ori` prelude.

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
        layout::{Affine, Align, AlignItems, Axis, Justify, Padding, Rect, Size, Space},
        log::*,
        rebuild::Rebuild,
        style::{
            builtin::*, em, set_style, set_text_size, set_theme, style, styled, Key, Palette, Theme,
        },
        text::{
            FontFamily, FontSource, FontStretch, FontStyle, FontWeight, Glyph, Glyphs, TextAlign,
            TextWrap,
        },
        transition::{ease, linear, Transition, TransitionCurve},
        view::{
            any, AnyView, BoxedView, BuildCx, ContentSequence, DrawCx, EventCx, LayoutCx,
            RebuildCx, SequenceState, State, View, ViewContent, ViewSequence, ViewState,
        },
        views::*,
        window::{Cursor, Window, WindowDescriptor, WindowId},
    };

    #[cfg(feature = "image")]
    pub use ori_core::image::image;
}
