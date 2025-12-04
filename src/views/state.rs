use std::{marker::PhantomData, mem::ManuallyDrop, ptr};

use crate::{Action, Event, View, ViewMarker};

/// [`View`] that maps one type of data to another.
pub fn map<C, T, U, E>(
    contents: impl View<C, U, Element = E>,
    map: impl FnMut(&mut T, &mut dyn FnMut(&mut U)),
) -> impl View<C, T, Element = E>
where
    T: ?Sized,
{
    Map::new(contents, map)
}

/// [`View`] that attaches extra `data` to it's contents.
pub fn with<C, T, U, V>(
    init: impl FnOnce(&mut T) -> U + 'static,
    build: impl FnOnce(&mut T, &mut U) -> V + 'static,
) -> impl View<C, T, Element = V::Element>
where
    V: View<C, (T, U)>,
{
    With::new(init, build)
}

/// [`View`] that attaches extra `data` using it's [`Default`] to it's contents.
pub fn with_default<C, T, U, V>(
    build: impl FnOnce(&mut T, &mut U) -> V + 'static,
) -> impl View<C, T, Element = V::Element>
where
    U: Default,
    V: View<C, (T, U)>,
{
    With::new(|_: &mut T| Default::default(), build)
}

/// [`View`] that maps one type of data to another.
#[must_use]
pub struct Map<F, U, V> {
    contents: V,
    map:      F,
    marker:   PhantomData<fn(&U)>,
}

impl<F, U, V> Map<F, U, V> {
    /// Create a [`Map`].
    pub fn new<T>(contents: V, map: F) -> Self
    where
        T: ?Sized,
        F: FnMut(&mut T, &mut dyn FnMut(&mut U)),
    {
        Self {
            contents,
            map,
            marker: PhantomData,
        }
    }
}

impl<F, U, V> ViewMarker for Map<F, U, V> {}
impl<C, T, U, V, F> View<C, T> for Map<F, U, V>
where
    T: ?Sized,
    V: View<C, U>,
    F: FnMut(&mut T, &mut dyn FnMut(&mut U)),
{
    type Element = V::Element;
    type State = V::State;

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let mut state = None;

        (self.map)(data, &mut |data| {
            state = Some(self.contents.build(cx, data));
        });

        state.expect("map calls lens")
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        let mut called = false;

        (self.map)(data, &mut |data| {
            self.contents.rebuild(
                element,
                state,
                cx,
                data,
                &mut old.contents,
            );

            called = true;
        });
    }

    fn teardown(&mut self, element: Self::Element, state: Self::State, cx: &mut C, data: &mut T) {
        let mut called = false;

        let mut element = Some(element);
        let mut state = Some(state);

        (self.map)(data, &mut |data| {
            if let (Some(element), Some(state)) = (element.take(), state.take()) {
                self.contents.teardown(element, state, cx, data);
                called = true;
            }
        });
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        let mut action = None;

        (self.map)(data, &mut |data| {
            action = Some(self.contents.event(element, state, cx, data, event));
        });

        action.unwrap_or(Action::new())
    }
}

/// [`View`] that attaches extra `data` to it's contents.
pub struct With<F, G> {
    init:  Option<F>,
    build: Option<G>,
}

impl<F, G> With<F, G> {
    /// Create a [`With`].
    pub fn new(init: F, build: G) -> Self {
        Self {
            init:  Some(init),
            build: Some(build),
        }
    }
}

impl<F, G> ViewMarker for With<F, G> {}
impl<F, G, C, T, U, V> View<C, T> for With<F, G>
where
    F: FnOnce(&mut T) -> U,
    G: FnOnce(&mut T, &mut U) -> V,
    V: View<C, (T, U)>,
{
    type Element = V::Element;
    type State = (U, V, V::State);

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let init = self.init.take().unwrap();
        let build = self.build.take().unwrap();

        let mut with = init(data);
        let mut view = build(data, &mut with);

        let (element, state) = {
            let mut data_with = DataWith::new(data, &mut with);
            view.build(cx, &mut data_with.data_with)
        };

        (element, (with, view, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (with, view, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        _old: &mut Self,
    ) {
        let build = self.build.take().unwrap();
        let mut new_view = build(data, with);

        let mut data_with = DataWith::new(data, with);
        new_view.rebuild(
            element,
            state,
            cx,
            &mut data_with.data_with,
            view,
        );

        *view = new_view;
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (mut with, mut view, state): Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        let mut data_with = DataWith::new(data, &mut with);
        view.teardown(
            element,
            state,
            cx,
            &mut data_with.data_with,
        );
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (with, view, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        let mut data_with = DataWith::new(data, with);
        view.event(
            element,
            state,
            cx,
            &mut data_with.data_with,
            event,
        )
    }
}

struct DataWith<'a, T, U> {
    data:      *mut T,
    with:      *mut U,
    data_with: ManuallyDrop<(T, U)>,
    marker:    PhantomData<(&'a mut T, &'a mut U)>,
}

impl<'a, T, U> DataWith<'a, T, U> {
    fn new(data: &'a mut T, with: &'a mut U) -> Self {
        let data = data as *mut T;
        let with = with as *mut U;

        let data_with = unsafe { (ptr::read(data), ptr::read(with)) };

        Self {
            data,
            with,
            data_with: ManuallyDrop::new(data_with),
            marker: PhantomData,
        }
    }
}

impl<T, U> Drop for DataWith<'_, T, U> {
    fn drop(&mut self) {
        unsafe {
            let (data, with) = ptr::read(&*self.data_with);
            ptr::write(self.data, data);
            ptr::write(self.with, with);
        }
    }
}
