use std::{marker::PhantomData, mem::ManuallyDrop, ptr};

use crate::{Action, Element, Message, Mut, View, ViewMarker};

/// [`View`] that maps one type of data to another.
pub fn map<C, T, U, E>(
    contents: impl View<C, U, Element = E>,
    map: impl FnMut(&mut T, &mut dyn FnMut(&mut U)),
) -> impl View<C, T, Element = E>
where
    T: ?Sized,
    E: Element,
{
    Map::new(contents, map)
}

/// [`View`] that maps one type of data to two types of data.
pub fn map_with<C, T, U, V, E>(
    contents: impl View<C, (U, V), Element = E>,
    mut map: impl FnMut(&mut T, &mut dyn FnMut(&mut U, &mut V)),
) -> impl View<C, T, Element = E>
where
    T: ?Sized,
    E: Element,
{
    Map::new(contents, move |data, outer| {
        map(data, &mut |with, data| {
            with_data(with, data, |with_data| {
                outer(with_data);
            });
        });
    })
}

/// [`View`] that attaches extra `data` to its contents.
pub fn with<C, T, U, V>(
    init: impl FnOnce(&T) -> U + 'static,
    build: impl FnOnce(&U, &T) -> V + 'static,
) -> impl View<C, T, Element = V::Element>
where
    V: View<C, (U, T)>,
{
    With::new(init, build)
}

/// [`View`] that attaches extra `data` using its [`Default`] to its contents.
pub fn with_default<C, T, U, V>(
    build: impl FnOnce(&U, &T) -> V + 'static,
) -> impl View<C, T, Element = V::Element>
where
    U: Default,
    V: View<C, (U, T)>,
{
    With::new(|_: &T| Default::default(), build)
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
    type State = (F, V::State);

    fn build(mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let mut contents = Some(self.contents);
        let mut state = None;

        (self.map)(data, &mut |data| {
            if let Some(contents) = contents.take() {
                state = Some(contents.build(cx, data));
            }
        });

        let (element, state) = state.expect("map should always call its `map`");
        (element, (self.map, state))
    }

    fn rebuild(
        mut self,
        element: Mut<'_, Self::Element>,
        (map, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        let mut contents = Some(self.contents);
        let mut element = Some(element);

        (self.map)(data, &mut |data| {
            if let Some(contents) = contents.take()
                && let Some(element) = element.take()
            {
                contents.rebuild(element, state, cx, data);
            }
        });

        *map = self.map;
    }

    fn message(
        element: Mut<'_, Self::Element>,
        (map, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        if message.is_taken() {
            return Action::new();
        }

        let mut action = None;
        let mut element = Some(element);

        map(data, &mut |data| {
            if let Some(element) = element.take() {
                action.replace(V::message(
                    element, state, cx, data, message,
                ));
            }
        });

        action.unwrap_or(Action::new())
    }

    fn teardown(element: Self::Element, (_, state): Self::State, cx: &mut C) {
        V::teardown(element, state, cx);
    }
}

/// [`View`] that attaches extra `data` to its contents.
pub struct With<F, G> {
    init:  F,
    build: G,
}

impl<F, G> With<F, G> {
    /// Create a [`With`].
    pub fn new(init: F, build: G) -> Self {
        Self { init, build }
    }
}

impl<F, G> ViewMarker for With<F, G> {}
impl<F, G, C, T, U, V> View<C, T> for With<F, G>
where
    F: FnOnce(&T) -> U,
    G: FnOnce(&U, &T) -> V,
    V: View<C, (U, T)>,
{
    type Element = V::Element;
    type State = (U, V::State);

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let mut with = (self.init)(data);
        let view = (self.build)(&with, data);

        let (element, state) = with_data(&mut with, data, |data_with| {
            view.build(cx, data_with)
        });

        (element, (with, state))
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        (with, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        let view = (self.build)(with, data);

        with_data(with, data, |data_with| {
            view.rebuild(element, state, cx, data_with);
        });
    }

    fn message(
        element: Mut<'_, Self::Element>,
        (with, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        with_data(with, data, |data_with| {
            V::message(element, state, cx, data_with, message)
        })
    }

    fn teardown(element: Self::Element, (_, state): Self::State, cx: &mut C) {
        V::teardown(element, state, cx);
    }
}

fn with_data<T, U, V>(data: &mut T, with: &mut U, f: impl FnOnce(&mut (T, U)) -> V) -> V {
    unsafe {
        let mut data_with = DataWith::new(data, with);
        f(&mut data_with.data_with)
    }
}

struct DataWith<'a, T, U> {
    data:      *mut T,
    with:      *mut U,
    data_with: ManuallyDrop<(T, U)>,
    marker:    PhantomData<(&'a mut T, &'a mut U)>,
}

impl<'a, T, U> DataWith<'a, T, U> {
    /// Do not use directly, use [`with_data`].
    ///
    /// # Safety
    /// - [`Self`] must be dropped, it cannot be forgotten.
    unsafe fn new(data: &'a mut T, with: &'a mut U) -> Self {
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
