mod align;
mod box_layout;
mod button;
mod class;
mod entry;
mod expand;
mod frame;
mod label;

pub use align::*;
pub use box_layout::*;
pub use button::*;
pub use class::*;
pub use entry::*;
pub use expand::*;
pub use frame::*;
pub use label::*;

use gtk4::glib::object::{Cast, IsA};

use crate::Context;

pub type AnyView<T> = Box<dyn ori::AnyView<Context, gtk4::Widget, T>>;

pub trait View<T>: ori::View<Context, T, Element: IsA<gtk4::Widget>> {}

impl<T, V> View<T> for V
where
    V: ori::View<Context, T>,
    V::Element: IsA<gtk4::Widget>,
{
}

impl<T> ori::Super<Context, T> for gtk4::Widget
where
    T: IsA<gtk4::Widget>,
{
    fn upcast(_cx: &mut Context, sub: T) -> Self {
        sub.upcast()
    }

    fn downcast<O>(&mut self, f: impl FnOnce(&mut T) -> O) -> O {
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
