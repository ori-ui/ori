#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub use ori_macro::{main, reloadable};

pub mod core {
    //! Ori [`core`](ori_core) module.

    pub use ori_core::*;
}

#[doc(hidden)]
#[cfg(feature = "reload")]
pub use ori_reload as reload;

#[cfg(feature = "font-awesome")]
pub mod font_awesome {
    //! Ori [`font-awesome`](ori_font_awesome) integration.

    pub use ori_font_awesome::*;
}

#[cfg(feature = "shell")]
pub use ori_shell::{launch, launch_simple};

pub mod prelude {
    //! Convenient imports for Ori.

    pub use ori_app::{App, AppBuilder, AppCommand, Delegate, DelegateCx};

    pub use ori_core::{
        canvas::{
            hex, hsl, hsla, okhsl, okhsla, okhsv, okhsva, oklab, oklaba, oklch, oklcha, rgb, rgba,
            BlendMode, BorderRadius, BorderWidth, Canvas, Color, Curve, FillRule, LineCap,
            LineJoin, Paint, Pattern, Shader, Stroke,
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
        style::{palette, style, style_or, styled, try_style, Palette, Style, Styles},
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

    pub use ori_macro::{desktop, mobile, Build};

    #[cfg(feature = "font-awesome")]
    pub use ori_font_awesome as fa;

    #[cfg(feature = "image")]
    pub use ori_core::include_image;
}
