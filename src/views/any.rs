use crate::{AnyView, BaseElement, Super, View};

/// Type erased [`View`].
#[must_use]
pub fn any<C, T, V>(view: V) -> Box<dyn AnyView<C, T, C::Element>>
where
    C: BaseElement,
    V: View<C, T> + 'static,
    V::State: 'static,
    C::Element: Super<C, V::Element>,
{
    Box::new(view)
}
