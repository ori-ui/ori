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

pub use app::App;
pub use context::Context;
pub use palette::Palette;
pub use view::{Effect, View};

pub mod prelude {
    pub use ike::{
        BorderWidth, Color, CornerRadius, FontStretch, FontStyle, FontWeight, Padding, TextAlign,
        TextWrap,
        widgets::{NewlineBehaviour, SubmitBehaviour},
    };

    pub use ori::{Action, Event, Proxy, ViewId, views::*};

    pub use crate::{App, Effect, View, ViewSeq, views::*};
}
