use crate::{AnyView, Base, Is, View};

/// Type erased [`View`].
#[must_use]
pub fn any<'a, C, T, V>(view: V) -> Box<dyn AnyView<C, T, C::Element> + 'a>
where
    C: Base,
    V: View<C, T> + 'a,
    V::State: 'static,
    V::Element: Is<C, C::Element>,
{
    Box::new(view)
}
