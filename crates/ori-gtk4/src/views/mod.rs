mod button;
mod checkbox;
mod entry;
mod frame;
mod label;
mod line;
mod picture;
mod popover;
mod properties;
mod window;

pub use button::{Button, button};
pub use checkbox::{Checkbox, checkbox};
pub use entry::{Entry, entry};
pub use frame::{Frame, frame};
pub use label::{Ellipsize, Label, Wrap, label};
pub use line::{Line, hline, line, vline};
pub use picture::{ImageSource, Picture, picture};
pub use popover::{Popover, PopoverCommand, popover};
pub use properties::{Align, Prop, WithProp};
pub use window::{Window, window};

#[cfg(feature = "layer-shell")]
pub use window::{Exclusive, Layer};

#[cfg(feature = "adwaita")]
mod clamp;

#[cfg(feature = "adwaita")]
pub use clamp::{Clamp, clamp, clamp_height, clamp_width};

use gtk4::{
    glib::object::{Cast, IsA},
    prelude::WidgetExt,
};

use crate::{AnyView, View};

fn is_parent(parent: &impl IsA<gtk4::Widget>, child: &impl IsA<gtk4::Widget>) -> bool {
    Some(parent.upcast_ref()) == child.upcast_ref().parent().as_ref()
}

#[must_use]
pub fn any<T, V>(view: V) -> AnyView<T>
where
    V: View<T> + 'static,
    V::State: 'static,
{
    Box::new(view)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl From<Axis> for gtk4::Orientation {
    fn from(axis: Axis) -> Self {
        match axis {
            Axis::Horizontal => gtk4::Orientation::Horizontal,
            Axis::Vertical => gtk4::Orientation::Vertical,
        }
    }
}
