/// Trait for defining subtype relations between [`View::Element`](crate::View::Element)s.
pub trait Super<C, S>: Element<C>
where
    S: Element<C>,
{
    /// Replace this element with another.
    fn replace(cx: &mut C, this: Mut<C, Self>, other: S) -> Self;

    /// Upcast sub type `S` to the super type `Self`.
    fn upcast(cx: &mut C, sub: S) -> Self;

    /// Downcast self to the sub type `S`.
    ///
    /// This is expected to panic when `self` is not an instance of `S`.
    fn downcast(self) -> S;

    /// Downcast self to the sub type `S`, and call `f` with it.
    ///
    /// This is expected to panic when `self` is not an instance of `S`.
    fn downcast_with<T>(this: Mut<C, Self>, f: impl FnOnce(Mut<C, S>) -> T) -> T;
}

/// An element maintained by a [`View`](crate::View).
pub trait Element<C> {
    /// A handle to the element through, e.g. a lock guard.
    type Mut<'a>
    where
        Self: 'a;
}

/// A shorthand for [`Element::Mut`].
pub type Mut<'a, C, T> = <T as Element<C>>::Mut<'a>;

/// Not an element.
#[derive(Debug)]
pub struct NoElement;

impl<C> Super<C, NoElement> for NoElement {
    fn replace(_cx: &mut C, _this: Mut<C, Self>, _other: NoElement) -> Self {
        Self
    }

    fn upcast(_cx: &mut C, _sub: NoElement) -> Self {
        NoElement
    }

    fn downcast(self) -> NoElement {
        NoElement
    }

    fn downcast_with<T>(_this: Mut<C, Self>, f: impl FnOnce(Mut<'_, C, Self>) -> T) -> T {
        f(())
    }
}

impl<C> Element<C> for NoElement {
    type Mut<'a> = ();
}
