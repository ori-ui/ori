use crate::Element;

/// A context that supportes teleporting [`Element`]s.
///
/// Teleporting works by [`Split`]ting the teleported [`Element`] into a `left` and a `right` part.
/// The `left` part is an immutable proxy of that is sent to the [`portal`](crate::views::portal),
/// the `right` part is kept by the [`teleport`](crate::views::teleport) for mutation. It is worth
/// noting that `right` may be torn down before `left` and vice versa and a sound implementation
/// should be able to handle that.
pub trait Teleportable {
    /// The left part of the split [`Element`].
    type Left: Element + 'static;
}

/// An [`Element`] that can be split into a `left` and `right` part, see [`Teleportable`] for more
/// information.
pub trait Split<T>: Teleportable
where
    T: Element,
{
    /// The right part of the split [`Element`].
    type Right;

    /// Split `self` into a `left` and `right` part.
    fn split(cx: &mut Self, widget: T) -> (Self::Left, Self::Right);

    /// Get a [`Mut`] of the underlying [`Element`].
    fn with_mut<'a, U>(
        right: &'a mut Self::Right,
        cx: &mut Self,
        f: impl FnOnce(&mut Self, T::Mut<'a>) -> U,
    ) -> U;

    /// Teardown `right` returning the underlying [`Element`].
    fn teardown(right: Self::Right, cx: &mut Self) -> T;
}
