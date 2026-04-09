use std::collections::HashMap;

use crate::{
    Action, Element, Elements, Is, Message, Mut, Provider, Proxied, Proxy, Split, Teleportable,
    Tracker, View, ViewId, ViewMarker, ViewSeq,
};

/// A [`ViewSeq`] that receives its [`Element`]s from [`teleport`].
pub fn portal(view_id: ViewId) -> Portal {
    Portal::new(view_id)
}

/// An [`Effect`](crate::Effect) that sends its contents to a [`portal`].
pub fn teleport<V>(portal: ViewId, contents: V) -> Teleport<V> {
    Teleport::new(portal, contents)
}

/// A [`ViewSeq`] that receives its [`Element`]s from [`teleport`].
pub struct Portal {
    view_id: ViewId,
}

impl Portal {
    /// Create new [`Portal`].
    pub fn new(view_id: ViewId) -> Self {
        Self { view_id }
    }
}

/// An [`Effect`](crate::Effect) that sends its contents to a [`portal`].
pub struct Teleport<V> {
    contents: V,
    portal:   ViewId,
}

impl<V> Teleport<V> {
    /// Create new [`Teleport`].
    pub fn new(portal: ViewId, contents: V) -> Self {
        Self { contents, portal }
    }
}

enum PortalMessage {
    Open(ViewId),
    Close(ViewId),
}

struct Lefts<P>(HashMap<ViewId, P>);
type Right<C, E> = <E as Split<C>>::Right;

impl<P> Default for Lefts<P> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

pub struct PortalState {
    view_id: ViewId,
    views:   Vec<ViewId>,
}

impl<C, T, E> ViewSeq<C, T, E> for Portal
where
    C: Tracker + Provider + Teleportable,
    E: Element,
    C::Left: Is<C, E>,
{
    type State = PortalState;

    fn seq_build(
        self,
        _elements: &mut impl Elements<C, E>,
        cx: &mut C,
        _data: &mut T,
    ) -> Self::State {
        if cx.get::<Lefts<C::Left>>().is_none() {
            let positives = Lefts::<C::Left>::default();
            cx.push(Box::new(positives));
        }

        cx.register(self.view_id);

        PortalState {
            view_id: self.view_id,
            views:   Vec::new(),
        }
    }

    fn seq_rebuild(
        self,
        elements: &mut impl Elements<C, E>,
        state: &mut Self::State,
        cx: &mut C,
        _data: &mut T,
    ) {
        for _ in &state.views {
            elements.next(cx);
        }
    }

    fn seq_message(
        elements: &mut impl Elements<C, E>,
        state: &mut Self::State,
        cx: &mut C,
        _data: &mut T,
        message: &mut Message,
    ) -> Action {
        match message.take(state.view_id) {
            Some(PortalMessage::Open(view_id)) => {
                for _ in &state.views {
                    elements.next(cx);
                }

                if let Some(positives) = cx.get_mut::<Lefts<C::Left>>()
                    && let Some(positive) = positives.0.remove(&view_id)
                {
                    let element = C::Left::upcast(cx, positive);
                    elements.insert(cx, element);
                    state.views.push(view_id);
                }

                Action::new()
            }

            Some(PortalMessage::Close(view_id)) => {
                state.views.retain(|view| {
                    if *view == view_id {
                        elements.remove(cx);
                        false
                    } else {
                        elements.next(cx);
                        true
                    }
                });

                Action::new()
            }

            None => {
                for _ in &state.views {
                    elements.next(cx);
                }

                Action::new()
            }
        }
    }

    fn seq_teardown(elements: &mut impl Elements<C, E>, state: Self::State, cx: &mut C) {
        for _ in &state.views {
            elements.remove(cx);
        }

        cx.unregister(state.view_id);
    }
}

impl<V> ViewMarker for Teleport<V> {}
impl<C, T, V> View<C, T> for Teleport<V>
where
    C: Tracker + Provider + Proxied + Teleportable,
    V: View<C, T>,
    V::Element: Split<C>,
{
    type Element = ();
    type State = TeleportState<C, T, V>;

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let (element, state) = self.contents.build(cx, data);
        let (shadow, negative) = element.split(cx);

        let view_id = ViewId::next();
        cx.register(view_id);

        if cx.get::<Lefts<C::Left>>().is_none() {
            let shadows = Lefts::<C::Left>::default();
            cx.push(Box::new(shadows));
        }

        if let Some(shadows) = cx.get_mut::<Lefts<C::Left>>() {
            shadows.0.insert(view_id, shadow);
        }

        cx.proxy().message(Message::new(
            PortalMessage::Open(view_id),
            self.portal,
        ));

        let state = TeleportState {
            view_id,
            portal: self.portal,
            right: negative,
            state,
        };

        ((), state)
    }

    fn rebuild(
        self,
        _element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        self.contents.rebuild(
            V::Element::as_mut(&mut state.right, cx),
            &mut state.state,
            cx,
            data,
        );

        #[cfg(feature = "tracing")]
        if state.portal != self.portal {
            tracing::error!("`teleport` does not support changing `portal`");
        }
    }

    fn message(
        _element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        V::Element::message(&mut state.right, cx, message);

        V::message(
            V::Element::as_mut(&mut state.right, cx),
            &mut state.state,
            cx,
            data,
            message,
        )
    }

    fn teardown(_element: Self::Element, state: Self::State, cx: &mut C) {
        cx.proxy().message(Message::new(
            PortalMessage::Close(state.view_id),
            state.portal,
        ));

        let element = V::Element::teardown(state.right, cx);
        V::teardown(element, state.state, cx);
        cx.unregister(state.view_id);
    }
}

pub struct TeleportState<C, T, V>
where
    C: Teleportable,
    V: View<C, T>,
    V::Element: Split<C>,
{
    view_id: ViewId,
    portal:  ViewId,
    right:   Right<C, V::Element>,
    state:   V::State,
}
