/// Trait for defining subtype relations between [`View::Element`](crate::View::Element)s.
pub trait Sub<C, S>: Element<C> + Sized
where
    S: Element<C>,
{
    /// Replace this element with another.
    fn replace(cx: &mut C, other: Mut<C, S>, this: Self) -> S;

    /// Upcast this to the base.
    fn upcast(cx: &mut C, this: Self) -> S;

    /// Try to downcast a base element to the specific [`Self`].
    fn downcast(cx: &mut C, this: S) -> Option<Self>;

    /// Try to downcast a mutable base element to the specific [`Self`].
    fn downcast_mut<T>(
        cx: &mut C,
        this: Mut<C, S>,
        f: impl FnOnce(&mut C, Mut<C, Self>) -> T,
    ) -> Option<T>;
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

impl<C> Sub<C, ()> for () {
    fn replace(_cx: &mut C, _other: Mut<C, Self>, _this: Self) -> Self {}

    fn upcast(_cx: &mut C, _this: Self) -> Self {}

    fn downcast(_cx: &mut C, _this: Self) -> Option<Self> {
        Some(())
    }

    fn downcast_mut<T>(
        cx: &mut C,
        _this: Mut<C, ()>,
        f: impl FnOnce(&mut C, Mut<C, Self>) -> T,
    ) -> Option<T> {
        Some(f(cx, ()))
    }
}

impl<C> Element<C> for () {
    type Mut<'a> = ();
}
