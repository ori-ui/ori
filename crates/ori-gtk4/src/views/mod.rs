mod align;
mod button;
mod checkbox;
mod class;
mod entry;
mod expand;
mod focusable;
mod frame;
mod label;
mod line;
mod size;

pub use align::{Align, Alignment, align, center, halign, valign};
pub use button::{Button, button};
pub use checkbox::{Checkbox, checkbox};
pub use class::{Class, class};
pub use entry::{Entry, entry};
pub use expand::{Expand, expand, hexpand, shrink, vexpand};
pub use frame::{Frame, frame};
pub use label::{Label, label};
pub use line::{Line, hline, line, vline};
pub use size::{Size, height, max_width, min_height, min_width, size, width};

use gtk4::{
    glib::object::{Cast, IsA},
    prelude::WidgetExt,
};

use crate::{AnyView, View};

fn is_parent(
    parent: &impl IsA<gtk4::Widget>,
    child: &impl IsA<gtk4::Widget>,
) -> bool {
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
