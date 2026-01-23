use crate::{AnyView, Base, Sub, View};

/// Type erased [`View`].
#[must_use]
pub fn any<C, T, V>(view: V) -> Box<dyn AnyView<C, T, C::Element>>
where
    C: Base,
    V: View<C, T> + 'static,
    V::State: 'static,
    V::Element: Sub<C, C::Element>,
{
    Box::new(view)
}
