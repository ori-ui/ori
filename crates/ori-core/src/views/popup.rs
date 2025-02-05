use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Affine, Size, Space},
    view::{Pod, PodState, View},
};

/// Create a new popup view.
pub fn popup<V, P>(content: V, popup: P) -> Popup<V, P> {
    Popup::new(content, popup)
}

/// A view that can popup.
pub struct Popup<V, P> {
    content: Pod<V>,
    popup: Pod<P>,
}

impl<V, P> Popup<V, P> {
    /// Create a new popup view.
    pub fn new(content: V, popup: P) -> Self {
        Self {
            content: Pod::new(content),
            popup: Pod::new(popup),
        }
    }
}

impl<T, V, P> View<T> for Popup<V, P>
where
    V: View<T>,
    P: View<T>,
{
    type State = (PodState<T, V>, PodState<T, P>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        cx.set_class("popup");
        cx.set_focusable(true);

        let content = self.content.build(cx, data);
        let popup = self.popup.build(cx, data);

        (content, popup)
    }

    fn rebuild(
        &mut self,
        (content, popup): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        self.content.rebuild(content, cx, data, &old.content);
        self.popup.rebuild(popup, cx, data, &old.popup);
    }

    fn event(
        &mut self,
        (content, popup): &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        let mut handled = self.content.event(content, cx, data, event);
        handled |= (self.popup).event_maybe(handled, popup, cx, data, event);

        if handled {
            return true;
        }

        match event {
            Event::PointerReleased(e) if e.clicked && content.is_hovered() => {
                cx.focus();
                cx.draw();

                true
            }

            Event::PointerReleased(e) if e.clicked && !popup.has_hovered() => {
                cx.set_focused(false);
                cx.draw();

                false
            }

            _ => false,
        }
    }

    fn layout(
        &mut self,
        (content, popup): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        let popup_space = Space::max(cx.window().size);
        let _ = self.popup.layout(popup, cx, data, popup_space);
        self.content.layout(content, cx, data, space)
    }

    fn draw(&mut self, (content, popup): &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        // draw the content first
        cx.layer(Affine::IDENTITY, None, Some(content.id()), |cx| {
            self.content.draw(content, cx, data);
        });

        if !cx.is_focused() && !popup.has_focused() {
            return;
        }

        // place the popup just below the content
        let offset = content.rect().bottom_center() - popup.rect().top_center();
        popup.translate(cx.transform() * offset);

        cx.overlay(0, |cx| {
            cx.layer(Affine::IDENTITY, None, Some(popup.id()), |cx| {
                self.popup.draw(popup, cx, data);
            });
        });
    }
}
