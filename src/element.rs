/// Trait for defining subtype relations between [`View::Element`](crate::View::Element)s.
pub trait Super<C, S> {
    /// Upcast sub type `S` to the super type `Self`.
    fn upcast(cx: &mut C, sub: S) -> Self;

    /// Downcast self to the sub type `S`.
    ///
    /// This is expected to panic when `self` is not an instance of `S`.
    fn downcast(self) -> S;

    /// Downcast self to the sub type `S`, and call `f` with it.
    ///
    /// This is expected to panic when `self` is not an instance of `S`.
    fn downcast_with<T>(&mut self, f: impl FnOnce(&mut S) -> T) -> T;
}

/// Not an element.
pub struct NoElement;

impl<C> Super<C, NoElement> for NoElement {
    fn upcast(_cx: &mut C, _sub: NoElement) -> Self {
        NoElement
    }

    fn downcast(self) -> NoElement {
        NoElement
    }

    fn downcast_with<T>(&mut self, f: impl FnOnce(&mut NoElement) -> T) -> T {
        f(&mut NoElement)
    }
}
