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

pub use align::{Align, Alignment, align, center, halign, valign};
pub use button::{Button, button};
pub use checkbox::{Checkbox, checkbox};
pub use class::{Class, class};
pub use entry::{Entry, entry};
pub use expand::{Expand, expand, hexpand, shrink, vexpand};
pub use frame::{Frame, frame};
pub use label::{Label, label};
pub use line::{Line, column, line, row};

use gtk4::{
    glib::object::{Cast, IsA},
    prelude::WidgetExt as _,
};

use crate::Context;

/// Type erased [`View`].
///
/// This is useful for building UI, based on controlflow.
pub type AnyView<T> = Box<dyn ori::AnyView<Context, gtk4::Widget, T>>;

pub trait View<T>: ori::View<Context, T, Element: IsA<gtk4::Widget>> {}
pub trait ViewSeq<T>: ori::ViewSeq<Context, gtk4::Widget, T> {}

impl<T, V> View<T> for V
where
    V: ori::View<Context, T>,
    V::Element: IsA<gtk4::Widget>,
{
}

impl<T, V> ViewSeq<T> for V where V: ori::ViewSeq<Context, gtk4::Widget, T> {}

impl<S> ori::Super<Context, S> for gtk4::Widget
where
    S: IsA<gtk4::Widget>,
{
    fn upcast(_cx: &mut Context, sub: S) -> Self {
        sub.upcast()
    }

    fn downcast(self) -> S {
        Cast::downcast(self).unwrap()
    }

    fn downcast_with<T>(&mut self, f: impl FnOnce(&mut S) -> T) -> T {
        let mut element = Cast::downcast(self.clone()).unwrap();
        let output = f(&mut element);
        *self = element.upcast();
        output
    }
}

#[must_use]
pub fn any<T, V>(view: V) -> AnyView<T>
where
    V: View<T> + 'static,
    V::State: 'static,
{
    Box::new(view)
}

fn is_parent(
    parent: &impl IsA<gtk4::Widget>,
    child: &impl IsA<gtk4::Widget>,
) -> bool {
    Some(parent.upcast_ref()) == child.upcast_ref().parent().as_ref()
}
