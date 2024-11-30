#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub use ori_macro::main;

pub mod core {
    //! Ori [`core`](ori_core) module.

    pub use ori_core::*;
}

pub mod app {
    //! Ori [`app`](ori_app) module.

    pub use ori_app::*;
}

#[cfg(feature = "shell")]
pub use ori_shell::{run, run_simple};

#[cfg(feature = "shell")]
pub mod shell {
    //! Ori [`shell`](ori_shell) module.

    pub use ori_shell::*;
}

pub mod log {
    //! Ori [`log`](ori_core::log) module.

    pub use ori_core::log::*;

    #[cfg(feature = "shell")]
    pub use ori_shell::install_logger as install;
}

pub mod prelude {
    //! Convenient imports for Ori.

    pub use ori_app::{App, AppBuilder, AppCommand, AppDelegate, DelegateCx};

    pub use ori_core::{
        canvas::{
            hex, hsl, hsla, hsv, hsva, okhsl, okhsla, okhsv, okhsva, oklab, oklaba, oklch, oklcha,
            rgb, rgba, BlendMode, BorderRadius, BorderWidth, Canvas, Color, Curve, FillRule, Paint,
            Pattern, Shader, Stroke, StrokeCap, StrokeJoin,
        },
        clipboard::Clipboard,
        command::CommandProxy,
        context::{BaseCx, BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
        event::{
            Code, Event, Key, KeyPressed, Modifiers, PointerButton, PointerId, PointerMoved,
            PointerPressed, PointerReleased, PointerScrolled, WindowCloseRequested,
        },
        image::{Image, ImageData, ImageId},
        layout::{
            pt, Affine, Align, Alignment, Axis, Justify, Matrix, Padding, Point, Rect, Size, Space,
            Vector, FILL,
        },
        log::{debug, error, info, trace, warn},
        rebuild::Rebuild,
        style,
        style::{comp, key, val, Style, Styled, Styles, Theme},
        text::{
            include_font, FontFamily, FontSource, FontStretch, FontStyle, FontWeight, Fonts,
            Paragraph, TextAlign, TextWrap,
        },
        transition::{ease, linear, Easing, Transition},
        view::{
            any, pod, AnyView, BoxedView, Pod, PodSeq, SeqState, State, View, ViewSeq, ViewState,
        },
        views::*,
        window::{Cursor, Pointer, Window, WindowId, WindowSizing},
    };

    pub use ori_macro::{desktop, is_desktop, is_mobile, is_web, mobile, web, Build, Styled};

    #[cfg(feature = "image")]
    pub use ori_core::include_image;
}

#[cfg(feature = "dylib")]
#[allow(unused_imports, clippy::single_component_path_imports)]
use ori_dylib;
