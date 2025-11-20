use ike::{AnyWidgetId, CastWidgetId};

use crate::Context;

pub trait View<T>: ori::View<Context, T, Element: ike::AnyWidgetId + ike::CastWidgetId> {}
pub trait Effect<T>: ori::Effect<Context, T> {}

impl<T, V> View<T> for V
where
    V: ori::View<Context, T>,
    V::Element: ike::AnyWidgetId + ike::CastWidgetId,
{
}
impl<T, V> Effect<T> for V where V: ori::Effect<Context, T> {}

impl<S> ori::Super<Context, S> for ike::WidgetId
where
    S: AnyWidgetId + CastWidgetId,
{
    fn upcast(_cx: &mut Context, sub: S) -> Self {
        sub.upcast()
    }

    fn downcast(self) -> S {
        S::downcast_unchecked(self)
    }

    fn downcast_with<T>(&mut self, f: impl FnOnce(&mut S) -> T) -> T {
        let mut id = self.downcast();
        let output = f(&mut id);
        *self = id.upcast();
        output
    }
}
