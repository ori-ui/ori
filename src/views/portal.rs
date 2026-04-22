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
            let left = Lefts::<C::Left>::default();
            cx.push(Box::new(left));
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

        if state.view_id != self.view_id {
            cx.unregister(state.view_id);
            cx.register(self.view_id);
            state.view_id = self.view_id;
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

                if let Some(lefts) = cx.get_mut::<Lefts<C::Left>>()
                    && let Some(left) = lefts.0.remove(&view_id)
                {
                    let element = C::Left::upcast(cx, left);
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
    C: Tracker + Provider + Proxied,
    C: Split<V::Element>,
    V: View<C, T>,
{
    type Element = ();
    type State = TeleportState<C, T, V>;

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let (element, state) = self.contents.build(cx, data);
        let (left, right) = C::split(cx, element);

        let view_id = ViewId::next();
        cx.register(view_id);

        if cx.get::<Lefts<C::Left>>().is_none() {
            let lefts = Lefts::<C::Left>::default();
            cx.push(Box::new(lefts));
        }

        if let Some(lefts) = cx.get_mut::<Lefts<C::Left>>() {
            lefts.0.insert(view_id, left);
        }

        cx.proxy().message(Message::new(
            PortalMessage::Open(view_id),
            self.portal,
        ));

        let state = TeleportState {
            view_id,
            portal: self.portal,
            right,
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
        C::with_mut(&mut state.right, cx, |cx, widget| {
            self.contents.rebuild(widget, &mut state.state, cx, data);
        });

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
        C::message(&mut state.right, cx, message);
        C::with_mut(&mut state.right, cx, |cx, widget| {
            V::message(
                widget,
                &mut state.state,
                cx,
                data,
                message,
            )
        })
    }

    fn teardown(_element: Self::Element, state: Self::State, cx: &mut C) {
        cx.proxy().message(Message::new(
            PortalMessage::Close(state.view_id),
            state.portal,
        ));

        let element = C::teardown(state.right, cx);
        V::teardown(element, state.state, cx);
        cx.unregister(state.view_id);
    }
}

pub struct TeleportState<C, T, V>
where
    C: Split<V::Element>,
    V: View<C, T>,
{
    view_id: ViewId,
    portal:  ViewId,
    right:   C::Right,
    state:   V::State,
}
