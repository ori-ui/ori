mod app;
mod context;
mod error;
mod key;
mod palette;
mod view;

#[cfg(feature = "vulkan")]
mod skia;

#[cfg(feature = "vulkan")]
mod vulkan;

pub mod views;
pub use ike::*;
pub use ori::*;
pub use tracing;

pub use app::App;
pub use context::Context;
pub use palette::Palette;
pub use view::{Effect, View};

pub mod prelude {
    pub use ike::{
        BorderWidth, Color, CornerRadius, FontStretch, FontStyle, FontWeight, Padding, Svg,
        TextAlign, TextWrap, Transition, include_svg,
        widgets::{Align, Axis, Fit, Justify, NewlineBehaviour, SubmitBehaviour},
    };

    pub use ori::{Action, Event, Proxy, ViewId, views::*};
    pub use tracing::{
        debug, debug_span, error, error_span, info, info_span, span, trace, trace_span, warn,
        warn_span,
    };

    pub use crate::{App, Effect, Palette, View, ViewSeq, views::*};
}
