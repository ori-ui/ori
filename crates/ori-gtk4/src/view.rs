use gtk4::glib::object::{Cast, IsA};

use crate::Context;

/// Type erased [`View`].
///
/// This is useful for building UI, based on controlflow.
pub type AnyView<T> = Box<dyn ori::AnyView<Context, T, gtk4::Widget>>;
pub type AnyEffect<T> = Box<dyn ori::AnyEffect<Context, T>>;

pub trait View<T>: ori::View<Context, T, Element: IsA<gtk4::Widget>> {}
pub trait ViewSeq<T>: ori::ViewSeq<Context, T, gtk4::Widget> {}

pub trait Effect<T>: ori::Effect<Context, T> {}

impl<T, V> View<T> for V
where
    V: ori::View<Context, T>,
    V::Element: IsA<gtk4::Widget>,
{
}

impl<T, V> ViewSeq<T> for V where V: ori::ViewSeq<Context, T, gtk4::Widget> {}

impl<T, V> Effect<T> for V where V: ori::Effect<Context, T> {}

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
