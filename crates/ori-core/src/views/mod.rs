//! The builtin views in Ori.

mod aligned;
mod animate;
mod build_handler;
mod button;
mod checkbox;
mod clickable;
mod collapsing;
mod color_picker;
mod constrain;
mod container;
mod draw_handler;
mod event_handler;
mod flex;
mod focus;
mod image;
mod memorize;
mod opaque;
mod pad;
mod painter;
mod rebuild_handler;
mod scroll;
mod slider;
mod stack;
mod text;
mod text_input;
mod tooltip;
mod transform;
mod trigger;
mod with_state;
mod wrap;
mod zstack;

pub use aligned::*;
pub use animate::*;
pub use build_handler::*;
pub use button::*;
pub use checkbox::*;
pub use clickable::*;
pub use collapsing::*;
pub use color_picker::*;
pub use constrain::*;
pub use container::*;
pub use draw_handler::*;
pub use event_handler::*;
pub use flex::*;
pub use focus::*;
pub use memorize::*;
pub use opaque::*;
pub use pad::*;
pub use painter::*;
pub use rebuild_handler::*;
pub use scroll::*;
pub use slider::*;
pub use stack::*;
pub use text::*;
pub use text_input::*;
pub use tooltip::*;
pub use transform::*;
pub use trigger::*;
pub use with_state::*;
pub use wrap::*;
pub use zstack::*;

#[cfg(test)]
#[allow(dead_code)]
mod testing {
    use std::collections::HashMap;

    use crate::{
        command::{CommandProxy, CommandReceiver, CommandWaker},
        context::{BaseCx, BuildCx, Contexts, DrawCx, EventCx, LayoutCx, RebuildCx},
        event::Event,
        layout::{Rect, Size, Space},
        view::{View, ViewState},
        window::Window,
    };

    pub struct ViewTester<T, V: View<T>> {
        pub state: V::State,
        pub view_state: ViewState,
        pub contexts: Contexts,
        pub command_rx: CommandReceiver,
        pub command_proxy: CommandProxy,
    }

    impl<T, V: View<T>> ViewTester<T, V> {
        pub fn new(view: &mut V, data: &mut T) -> Self {
            let waker = CommandWaker::new(|| {});

            let window = Window::new();

            let mut contexts = Contexts::new();
            contexts.insert(window);

            let (mut proxy, rx) = CommandProxy::new(waker);

            let mut view_state = ViewState::default();

            let mut base_cx = BaseCx::new(&mut contexts, &mut proxy);
            let mut build_cx = BuildCx::new(&mut base_cx, &mut view_state);

            let state = view.build(&mut build_cx, data);

            Self {
                state,
                view_state,
                contexts,
                command_rx: rx,
                command_proxy: proxy,
            }
        }

        pub fn rebuild(&mut self, view: &mut V, data: &mut T, old: &V) {
            let mut base_cx = BaseCx::new(&mut self.contexts, &mut self.command_proxy);
            let mut rebuild_cx = RebuildCx::new(&mut base_cx, &mut self.view_state);

            view.rebuild(&mut self.state, &mut rebuild_cx, data, old);
        }

        pub fn event(&mut self, view: &mut V, data: &mut T, event: &Event) -> bool {
            let mut needs_rebuild = false;

            let mut base_cx = BaseCx::new(&mut self.contexts, &mut self.command_proxy);
            let mut event_cx = EventCx::new(&mut base_cx, &mut self.view_state, &mut needs_rebuild);
            view.event(&mut self.state, &mut event_cx, data, event);

            needs_rebuild
        }

        pub fn layout(&mut self, view: &mut V, data: &mut T, space: Space) -> Size {
            let mut base_cx = BaseCx::new(&mut self.contexts, &mut self.command_proxy);
            let mut layout_cx = LayoutCx::new(&mut base_cx, &mut self.view_state);

            let size = view.layout(&mut self.state, &mut layout_cx, data, space);
            self.view_state.set_size(size);

            size
        }
    }

    pub fn test_layout<T>(view: &mut impl View<T>, data: &mut T, space: Space) -> SavedLayouts {
        let mut tester = ViewTester::new(view, data);
        tester.layout(view, data, space);
        tester.event(view, data, &Event::Update);
        tester.contexts.get_or_default::<SavedLayouts>().clone()
    }

    pub type SavedLayouts = HashMap<String, Rect>;

    pub fn save_layout<V>(content: V, name: impl Into<String>) -> LayoutSaver<V> {
        LayoutSaver::new(content, name)
    }

    pub struct LayoutSaver<V> {
        pub content: V,
        pub name: String,
    }

    impl<V> LayoutSaver<V> {
        pub fn new(content: V, name: impl Into<String>) -> Self {
            Self {
                content,
                name: name.into(),
            }
        }
    }

    impl<T, V: View<T>> View<T> for LayoutSaver<V> {
        type State = V::State;

        fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
            self.content.build(cx, data)
        }

        fn rebuild(
            &mut self,
            state: &mut Self::State,
            cx: &mut RebuildCx,
            data: &mut T,
            old: &Self,
        ) {
            self.content.rebuild(state, cx, data, &old.content);
        }

        fn event(
            &mut self,
            state: &mut Self::State,
            cx: &mut EventCx,
            data: &mut T,
            event: &Event,
        ) {
            self.content.event(state, cx, data, event);

            let layout_rect = cx.rect().transform(cx.transform());
            cx.context_or_default::<SavedLayouts>()
                .insert(self.name.clone(), layout_rect);
        }

        fn layout(
            &mut self,
            state: &mut Self::State,
            cx: &mut LayoutCx,
            data: &mut T,
            space: Space,
        ) -> Size {
            self.content.layout(state, cx, data, space)
        }

        fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
            self.content.draw(state, cx, data);
        }
    }
}
