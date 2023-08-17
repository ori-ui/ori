pub use ori_core as core;
pub use ori_wgpu as wgpu;

pub mod prelude {
    //! The `ori` prelude.

    pub use crate::core::math::*;
    pub use crate::core::views::{
        button, container, focus, hstack, pad, text, vstack, Button, Container, Focus, Stack, Text,
    };
    pub use crate::core::{
        any, button, ease, linear, style, Affine, Align, Axis, BaseCx, Canvas, Color, DrawCx,
        Event, EventCx, Key, LayoutCx, Mesh, Padding, Pod, PodSequence, PodSequenceState, PodState,
        Primitive, RebuildCx, Rect, Size, Space, Theme, View, ViewSequence, Window,
    };
    pub use crate::core::{hstack, vstack};

    pub use crate::wgpu::App;
}
