pub use ori_core as core;
pub use ori_wgpu as wgpu;

pub mod prelude {
    //! The `ori` prelude.

    pub use crate::core::math::*;
    pub use crate::core::tracing::{debug, error, info, trace, warn};
    pub use crate::core::views::{
        align, align_bottom, align_bottom_left, align_bottom_right, align_center, align_left,
        align_right, align_top, align_top_left, align_top_right, button, checkbox, container,
        focus, hstack, pad, text, text_input, vstack, Button, Checkbox, Container, Focus, Stack,
        Text, TextInput,
    };
    pub use crate::core::{
        any, builtin::*, ease, em, hex, hsl, hsla, linear, rgb, rgba, style, Affine, Align,
        AlignItems, Axis, BaseCx, BorderRadius, BorderWidth, BuildCx, Canvas, Color, Cursor, Curve,
        DrawCx, Event, EventCx, FontFamily, FontSource, FontStretch, FontStyle, FontWeight,
        Justify, Key, LayoutCx, Mesh, Padding, Palette, Pod, PodSequence, PodSequenceState,
        PodState, Primitive, Quad, Rebuild, RebuildCx, Rect, Size, Space, TextAlign, TextSection,
        TextWrap, Theme, Vertex, View, ViewSequence, Window,
    };
    pub use crate::core::{hstack, vstack};

    pub use crate::wgpu::App;
}
