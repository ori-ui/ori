use crate::{Action, Element, Elements, Message, ViewSeq};

/// A [`ViewSeq`] that can temporarily remove its contents while preserving its elements and state.
pub fn detach<V>(detached: bool, contents: V) -> Detach<V> {
    Detach::new(detached, contents)
}

/// A [`ViewSeq`] that can temporarily remove its contents while preserving its elements and state.
pub struct Detach<V> {
    detached: bool,
    contents: V,
}

impl<V> Detach<V> {
    /// Create new [`Detach`].
    pub fn new(detached: bool, contents: V) -> Self {
        Self { detached, contents }
    }
}

/// An element that can be [`detach`]ed.
pub trait Detachable<C>: Element {
    /// Get a [`Self::Mut`] from a mutable reference to `self`.
    fn as_detached<'a>(&'a mut self, cx: &mut C) -> Self::Mut<'a>;
}

pub struct DetachState<C, T, V, E>
where
    E: Detachable<C>,
    V: ViewSeq<C, T, E>,
{
    state:    V::State,
    count:    usize,
    scratch:  Vec<E>,
    detached: bool,
}

impl<C, T, V, E> ViewSeq<C, T, E> for Detach<V>
where
    E: Detachable<C>,
    V: ViewSeq<C, T, E>,
{
    type State = DetachState<C, T, V, E>;

    fn seq_build(
        self,
        elements: &mut impl Elements<C, E>,
        cx: &mut C,
        data: &mut T,
    ) -> Self::State {
        let mut scratch = Vec::new();

        match self.detached {
            true => {
                let mut elements = ScratchElements::new(&mut scratch);
                let state = self.contents.seq_build(&mut elements, cx, data);

                DetachState {
                    count: scratch.len(),
                    detached: self.detached,

                    state,
                    scratch,
                }
            }

            false => {
                let mut count = 0;
                let mut elements = TrackedElements::new(&mut count, elements);
                let state = self.contents.seq_build(&mut elements, cx, data);

                DetachState {
                    state,
                    count,
                    scratch,
                    detached: self.detached,
                }
            }
        }
    }

    fn seq_rebuild(
        self,
        elements: &mut impl Elements<C, E>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        match (state.detached, self.detached) {
            (true, true) => {
                V::seq_rebuild(
                    self.contents,
                    &mut ScratchElements::new(&mut state.scratch),
                    &mut state.state,
                    cx,
                    data,
                );

                state.count = state.scratch.len();
            }

            (false, false) => {
                V::seq_rebuild(
                    self.contents,
                    &mut TrackedElements::new(&mut state.count, elements),
                    &mut state.state,
                    cx,
                    data,
                );
            }

            (true, false) => {
                V::seq_rebuild(
                    self.contents,
                    &mut ScratchElements::new(&mut state.scratch),
                    &mut state.state,
                    cx,
                    data,
                );

                state.count = state.scratch.len();
                state.detached = false;

                for element in state.scratch.drain(..) {
                    elements.insert(cx, element);
                }
            }

            (false, true) => {
                for _ in 0..state.count {
                    if let Some(element) = elements.remove(cx) {
                        state.scratch.push(element);
                    }
                }

                V::seq_rebuild(
                    self.contents,
                    &mut ScratchElements::new(&mut state.scratch),
                    &mut state.state,
                    cx,
                    data,
                );

                state.count = state.scratch.len();
                state.detached = true;
            }
        }
    }

    fn seq_message(
        elements: &mut impl Elements<C, E>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        match state.detached {
            true => {
                let action = V::seq_message(
                    &mut ScratchElements::new(&mut state.scratch),
                    &mut state.state,
                    cx,
                    data,
                    message,
                );

                state.count = state.scratch.len();
                action
            }

            false => V::seq_message(
                &mut TrackedElements::new(&mut state.count, elements),
                &mut state.state,
                cx,
                data,
                message,
            ),
        }
    }

    fn seq_teardown(elements: &mut impl Elements<C, E>, mut state: Self::State, cx: &mut C) {
        match state.detached {
            true => V::seq_teardown(
                &mut ScratchElements::new(&mut state.scratch),
                state.state,
                cx,
            ),

            false => V::seq_teardown(elements, state.state, cx),
        }
    }
}

struct ScratchElements<'a, E> {
    index:   usize,
    scratch: &'a mut Vec<E>,
}

impl<'a, E> ScratchElements<'a, E> {
    fn new(scratch: &'a mut Vec<E>) -> Self {
        Self { index: 0, scratch }
    }
}

impl<'a, C, E> Elements<C, E> for ScratchElements<'a, E>
where
    E: Detachable<C>,
{
    fn index(&self) -> usize {
        self.index
    }

    fn next(&mut self, cx: &mut C) -> Option<E::Mut<'_>> {
        let element = self.scratch.get_mut(self.index)?;
        self.index += 1;

        Some(element.as_detached(cx))
    }

    fn insert(&mut self, _cx: &mut C, element: E) {
        self.scratch.insert(self.index, element);
        self.index += 1;
    }

    fn remove(&mut self, _cx: &mut C) -> Option<E> {
        if self.index >= self.scratch.len() {
            return None;
        }

        Some(self.scratch.remove(self.index))
    }

    fn swap(&mut self, _cx: &mut C, offset: usize) {
        self.scratch.swap(self.index, self.index + offset);
    }
}

struct TrackedElements<'a, T> {
    count:    &'a mut usize,
    elements: &'a mut T,
}

impl<'a, T> TrackedElements<'a, T> {
    fn new(count: &'a mut usize, elements: &'a mut T) -> Self {
        Self { count, elements }
    }
}

impl<'a, C, T, E> Elements<C, E> for TrackedElements<'a, T>
where
    T: Elements<C, E>,
    E: Element,
{
    fn index(&self) -> usize {
        self.elements.index()
    }

    fn next(&mut self, cx: &mut C) -> Option<E::Mut<'_>> {
        self.elements.next(cx)
    }

    fn insert(&mut self, cx: &mut C, element: E) {
        self.elements.insert(cx, element);
        *self.count += 1;
    }

    fn remove(&mut self, cx: &mut C) -> Option<E> {
        let element = self.elements.remove(cx)?;
        *self.count -= 1;
        Some(element)
    }

    fn swap(&mut self, cx: &mut C, offset: usize) {
        self.elements.swap(cx, offset);
    }
}
