use crate::{
    canvas::Canvas,
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::{Code, Event, KeyPressed},
    layout::{Size, Space, Vector},
    view::{Pod, State, View},
};

use super::{DebugTree, History};

#[derive(Debug)]
pub(super) struct DebugData {
    pub history: History,
    pub tree: DebugTree,
    pub is_open: bool,
}

pub(super) struct DebugView<V, D> {
    content: Pod<V>,
    #[allow(clippy::type_complexity)]
    builder: Option<Box<dyn FnOnce(&mut DebugData) -> D>>,
    debugger: Option<Pod<D>>,
}

impl<V, D> DebugView<V, D> {
    /// Create a new [`DebugView`].
    pub fn new(content: V, debugger: impl FnMut(&mut DebugData) -> D + 'static) -> Self {
        Self {
            content: Pod::new(content),
            builder: Some(Box::new(debugger)),
            debugger: Option::None,
        }
    }

    fn debugger(&mut self, data: &mut DebugData) -> &mut Pod<D> {
        if let Some(ref mut content) = self.debugger {
            return content;
        }

        let builder = self.builder.take().unwrap();
        self.debugger = Some(Pod::new(builder(data)));
        self.debugger.as_mut().unwrap()
    }
}

pub(super) struct DebugViewState<T, V: View<T>, D: View<DebugData>> {
    content: State<T, V>,
    debugger: State<DebugData, D>,
    debug_tree: Option<DebugTree>,
    is_open: bool,
}

impl<V: View<T>, D: View<DebugData>, T> View<T> for DebugView<V, D> {
    type State = DebugViewState<T, V, D>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let content = self.content.build(cx, data);

        let debug_tree = DebugTree::new();

        let mut data = DebugData {
            history: cx.remove_context::<History>().unwrap(),
            tree: debug_tree,
            is_open: false,
        };

        let debugger = self.debugger(&mut data).build(cx, &mut data);

        cx.insert_context(data.history);

        DebugViewState {
            content,
            debugger,
            debug_tree: Some(data.tree),
            is_open: data.is_open,
        }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        // if state isn't open, we don't need to build the debug tree
        if !state.is_open {
            (self.content).rebuild(&mut state.content, cx, data, &old.content);
            return;
        }

        cx.insert_context(state.debug_tree.take().unwrap());
        (self.content).rebuild(&mut state.content, cx, data, &old.content);

        // create the debug data
        let mut data = DebugData {
            history: cx.remove_context::<History>().unwrap(),
            tree: cx.remove_context::<DebugTree>().unwrap(),
            is_open: state.is_open,
        };

        let debugger = self.debugger(&mut data);
        let old_debugger = old.debugger.as_ref().unwrap();
        debugger.rebuild(&mut state.debugger, cx, &mut data, old_debugger);

        // decay the tree, this will remove any old nodes that haven't been rebuilt
        data.tree.decay();

        cx.insert_context(data.history);
        state.debug_tree = Some(data.tree);
        state.is_open = data.is_open;
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        if let Some(event) = event.get::<KeyPressed>() {
            if event.is(Code::I) && event.modifiers.ctrl && event.modifiers.shift {
                state.is_open = !state.is_open;
                cx.request_rebuild();
                cx.request_layout();
                cx.request_draw();
            }
        }

        if !state.is_open {
            (self.content).event(&mut state.content, cx, data, event);
            return;
        }

        cx.insert_context(state.debug_tree.take().unwrap());
        (self.content).event(&mut state.content, cx, data, event);

        let mut data = DebugData {
            history: cx.remove_context::<History>().unwrap(),
            tree: cx.remove_context::<DebugTree>().unwrap(),
            is_open: state.is_open,
        };

        (self.debugger(&mut data)).event(&mut state.debugger, cx, &mut data, event);

        cx.insert_context(data.history);
        state.debug_tree = Some(data.tree);

        if state.is_open != data.is_open {
            state.is_open = data.is_open;
            cx.request_rebuild();
            cx.request_layout();
        }
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        if !state.is_open {
            return (self.content).layout(&mut state.content, cx, data, space);
        }

        let mut debug_data = DebugData {
            history: cx.remove_context::<History>().unwrap(),
            tree: state.debug_tree.take().unwrap(),
            is_open: state.is_open,
        };

        let debugger = self.debugger(&mut debug_data);
        let size = debugger.layout(&mut state.debugger, cx, &mut debug_data, space.loosen());

        let offset = Vector::new(0.0, space.max.height - size.height);
        state.debugger.translate(offset);

        let content_space = space.shrink(Size::new(0.0, size.height));

        cx.insert_context(debug_data.history);
        cx.insert_context(debug_data.tree);
        state.is_open = debug_data.is_open;

        (self.content).layout(&mut state.content, cx, data, content_space);
        state.debug_tree = cx.remove_context::<DebugTree>();

        space.max
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        if !state.is_open {
            (self.content).draw(&mut state.content, cx, data, canvas);
            return;
        }

        let debug_tree = state.debug_tree.take().unwrap();
        let _tree_hash = debug_tree.fast_hash();
        cx.insert_context(debug_tree);

        (self.content).draw(&mut state.content, cx, data, canvas);

        let mut data = DebugData {
            history: cx.remove_context::<History>().unwrap(),
            tree: cx.remove_context::<DebugTree>().unwrap(),
            is_open: state.is_open,
        };

        (self.debugger(&mut data)).draw(&mut state.debugger, cx, &mut data, canvas);

        cx.insert_context(data.history);
        state.debug_tree = Some(data.tree);
        state.is_open = data.is_open;
    }
}
