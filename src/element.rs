/// A context with a common base element, that is [`Super`](crate::Super) to all elements in the
/// context.
pub trait Base: Sized {
    /// The base element.
    type Element: Element;
}

/// Trait for defining subtype relations between [`View::Element`](crate::View::Element)s.
pub trait Is<C, S>: Element + Sized + 'static
where
    S: Element,
{
    /// Replace this element with another.
    fn replace(cx: &mut C, other: Mut<'_, S>, this: Self) -> S;

    /// Upcast this to the base.
    fn upcast(cx: &mut C, this: Self) -> S;

    /// Try to downcast a base element to the specific [`Self`].
    fn downcast(this: S) -> Result<Self, S>;

    /// Try to downcast a mutable base element to the specific [`Self`].
    fn downcast_mut(this: S::Mut<'_>) -> Result<Self::Mut<'_>, S::Mut<'_>>;
}

/// An element maintained by a [`View`](crate::View).
pub trait Element {
    /// A handle to the element to mutate through, e.g. a lock guard.
    type Mut<'a>
    where
        Self: 'a;
}

/// A shorthand for [`Element::Mut`].
pub type Mut<'a, T> = <T as Element>::Mut<'a>;

impl<C> Is<C, ()> for () {
    fn replace(_cx: &mut C, _other: Mut<'_, Self>, _this: Self) -> Self {}

    fn upcast(_cx: &mut C, _this: Self) -> Self {}

    fn downcast(_this: ()) -> Result<Self, ()> {
        Ok(())
    }

    fn downcast_mut(_this: Mut<'_, ()>) -> Result<Mut<'_, Self>, Mut<'_, ()>> {
        Ok(())
    }
}

impl Element for () {
    type Mut<'a> = ();
}
