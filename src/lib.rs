#![warn(missing_docs)]

//! Ori is a cross-platform declarative UI framework for Rust, with a focus on
//! simplicity and performance.

pub use ori_core as core;
pub use ori_wgpu as wgpu;

pub mod prelude {
    //! The `ori` prelude.

    pub use crate::wgpu::App;

    pub use crate::core::math::*;
    pub use crate::core::tracing::{debug, error, info, trace, warn};
    pub use crate::core::views::{
        align, align_bottom, align_bottom_left, align_bottom_right, align_center, align_left,
        align_right, align_top, align_top_left, align_top_right, button, checkbox, container, flex,
        focus, hstack, pad, text, text_input, vstack, Button, Checkbox, Container, Flex, Focus,
        Stack, Text, TextInput,
    };
    pub use crate::core::{
        any, builtin::*, ease, em, hex, hsl, hsla, linear, rgb, rgba, style, Affine, Align,
        AlignItems, Axis, BaseCx, BorderRadius, BorderWidth, BuildCx, Canvas, Color, Command,
        ContentSequence, ContentSequenceState, Cursor, Curve, Delegate, DelegateCx, DrawCx, Event,
        EventCx, FontFamily, FontSource, FontStretch, FontStyle, FontWeight, Justify, Key,
        LayoutCx, Mesh, Padding, Palette, Primitive, Quad, Rebuild, RebuildCx, Rect, Size, Space,
        State, TextAlign, TextSection, TextWrap, Theme, Vertex, View, ViewContent, ViewSequence,
        Window,
    };
    pub use crate::core::{hstack, vstack};

    #[cfg(feature = "image")]
    pub use crate::core::image;
}
