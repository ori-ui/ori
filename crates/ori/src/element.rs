/// Trait for defining subtype relations between [`View::Element`](crate::View::Element)s.
pub trait Super<C, S> {
    fn upcast(cx: &mut C, sub: S) -> Self;

    fn downcast<T>(&mut self, f: impl FnOnce(&mut S) -> T) -> T;
}
