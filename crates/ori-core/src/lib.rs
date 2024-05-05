#![allow(clippy::module_inception)]
#![warn(missing_docs)]

//! Core library for the Ori UI framework.

pub mod canvas;
pub mod clipboard;
pub mod command;
pub mod context;
pub mod event;
pub mod image;
pub mod layout;
pub mod rebuild;
pub mod style;
pub mod text;
pub mod transition;
pub mod view;
pub mod window;

pub mod views;

pub use tracing as log;

pub mod prelude {
    //! Convenient imports for Ori.

    pub use crate::{
        canvas::{
            hex, hsl, hsla, oklab, oklaba, rgb, rgba, BlendMode, BorderRadius, BorderWidth, Canvas,
            Color, Curve, FillRule, LineCap, LineJoin, Paint, Shader, Stroke,
        },
        clipboard::Clipboard,
        command::CommandProxy,
        context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
        event::{
            CloseRequested, Code, Event, KeyPressed, Modifiers, PointerButton, PointerId,
            PointerMoved, PointerPressed, PointerReleased, PointerScrolled,
        },
        image::{Image, ImageData, ImageId},
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
        transition::{ease, linear, Easing, Transition},
        view::{
            any, pod, AnyView, BoxedView, Pod, PodSeq, SeqState, State, View, ViewSeq, ViewState,
        },
        views::*,
        window::{Cursor, Pointer, Window, WindowId, WindowSizing},
    };
}
