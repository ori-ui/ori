use crate::Context;

pub trait View<T>: ori::View<Context, T, Element = ike::WidgetId> {}
pub trait Effect<T>: ori::Effect<Context, T> {}

impl<T, V> View<T> for V where V: ori::View<Context, T, Element = ike::WidgetId> {}
impl<T, V> Effect<T> for V where V: ori::Effect<Context, T> {}
