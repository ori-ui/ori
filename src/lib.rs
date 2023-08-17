pub use ori_core as core;
pub use ori_wgpu as wgpu;

pub mod prelude {
    //! The `ori` prelude.

    pub use crate::core::math::*;
    pub use crate::core::views::{
        align, align_bottom, align_bottom_left, align_bottom_right, align_center, align_left,
        align_right, align_top, align_top_left, align_top_right, button, container, focus, hstack,
        pad, text, text_input, vstack, Button, Container, Focus, Stack, Text, TextInput,
    };
    pub use crate::core::{
        any, builtin::*, ease, linear, style, Affine, Align, AlignItems, Axis, BaseCx, BuildCx,
        Canvas, Color, DrawCx, Event, EventCx, Justify, Key, LayoutCx, Mesh, Padding, Pod,
        PodSequence, PodSequenceState, PodState, Primitive, Rebuild, RebuildCx, Rect, Size, Space,
        Theme, View, ViewSequence, Window,
    };
    pub use crate::core::{hstack, vstack};

    pub use crate::wgpu::App;
}
