use crate::{View, ViewSeq};

/// A [`View`] that has [`NoElement`] and can therefore only produce side-effects.
///
/// Implemented for all [`View`]s with an element of [`NoElement`].
pub trait Effect<C, T>: View<C, T, Element = ()> {}

/// A sequence of [`Effect`]s.
///
/// Implemented for all [`ViewSeq`]s with an element of [`NoElement`].
pub trait EffectSeq<C, T>: ViewSeq<C, T, ()> {}

impl<C, T, V> Effect<C, T> for V where V: View<C, T, Element = ()> {}
impl<C, T, V> EffectSeq<C, T> for V where V: ViewSeq<C, T, ()> {}
