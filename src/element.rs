/// A context with a common base element, that [`Is`] all elements in the context.
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
    fn replace(cx: &mut C, other: S::Mut<'_>, this: Self) -> S;

    /// Upcast this to the base.
    fn upcast(cx: &mut C, this: Self) -> S;

    /// Try to downcast a base element to the specific [`Self`].
    fn downcast(this: S) -> Result<Self, S>;

    /// Try to downcast a mutable base element to the specific [`Self`].
    fn downcast_mut(this: S::Mut<'_>) -> Result<Self::Mut<'_>, S::Mut<'_>>;
}

/// An element that is a subtype of another.
pub trait Sub<C, S>: Element + Sized + 'static
where
    S: Element,
{
    /// Replace this element with the sub-element.
    fn replace(cx: &mut C, this: Self::Mut<'_>, sub: S) -> Self;

    /// Upcast the sub-element to `Self`.
    fn upcast(cx: &mut C, sub: S) -> Self;

    /// Try to downcast `self` to the subtype.
    fn downcast(this: Self) -> Result<S, Self>;

    /// Try to downcast `Self::Mut` to the `S::Mut`.
    fn downcast_mut(this: Self::Mut<'_>) -> Result<S::Mut<'_>, Self::Mut<'_>>;
}

impl<C, S, T> Is<C, S> for T
where
    S: Sub<C, T>,
    T: Element + 'static,
{
    fn replace(cx: &mut C, other: S::Mut<'_>, this: Self) -> S {
        S::replace(cx, other, this)
    }

    fn upcast(cx: &mut C, this: Self) -> S {
        S::upcast(cx, this)
    }

    fn downcast(this: S) -> Result<Self, S> {
        S::downcast(this)
    }

    fn downcast_mut(this: S::Mut<'_>) -> Result<Self::Mut<'_>, S::Mut<'_>> {
        S::downcast_mut(this)
    }
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

impl<C> Sub<C, Self> for () {
    fn replace(_cx: &mut C, _this: Self::Mut<'_>, _sub: Self) -> Self {}

    fn upcast(_cx: &mut C, _sub: Self) -> Self {}

    fn downcast(_this: Self) -> Result<Self, Self> {
        Ok(())
    }

    fn downcast_mut(_this: Self::Mut<'_>) -> Result<Self::Mut<'_>, Self::Mut<'_>> {
        Ok(())
    }
}

impl Element for () {
    type Mut<'a> = ();
}
