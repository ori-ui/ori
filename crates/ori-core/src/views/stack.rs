use std::cell::Cell;

use ori_macro::Build;

use crate::{
    canvas::Canvas,
    event::{Event, RequestFocus, SwitchFocus},
    layout::{Align, Axis, Justify, Size, Space},
    rebuild::Rebuild,
    view::{
        AnyView, BuildCx, DrawCx, EventCx, LayoutCx, PodSeq, RebuildCx, SeqState, View, ViewSeq,
    },
};

pub use crate::{hstack, vstack};

/// Create a horizontal [`Stack`].
#[macro_export]
macro_rules! hstack {
    (for $iter:expr) => {
        $crate::views::hstack(
            <::std::vec::Vec<_> as ::std::iter::FromIterator<_>>::from_iter($iter)
        )
    };
    ($($child:expr),* $(,)?) => {
        $crate::views::hstack(($($child,)*))
    };
}

/// Create a vertical [`Stack`].
#[macro_export]
macro_rules! vstack {
    (for $iter:expr) => {
        $crate::views::vstack(
            <::std::vec::Vec<_> as ::std::iter::FromIterator<_>>::from_iter($iter)
        )
    };
    ($($child:expr),* $(,)?) => {
        $crate::views::vstack(($($child,)*))
    };
}

/// Create a horizontal [`Stack`].
pub fn hstack<V>(content: V) -> Stack<V> {
    Stack::hstack(content)
}

/// Create a vertical [`Stack`].
pub fn vstack<V>(content: V) -> Stack<V> {
    Stack::vstack(content)
}

/// Create a horizontal [`Stack`], with dynamic content.
pub fn hstack_any<'a, V>() -> Stack<Vec<Box<dyn AnyView<V> + 'a>>> {
    Stack::hstack_any()
}

/// Create a vertical [`Stack`], with dynamic content.
pub fn vstack_any<'a, V>() -> Stack<Vec<Box<dyn AnyView<V> + 'a>>> {
    Stack::vstack_any()
}

/// A view that stacks its content in a line.
#[derive(Build, Rebuild)]
pub struct Stack<V> {
    /// The content of the stack.
    #[build(ignore)]
    pub content: PodSeq<V>,
    /// The axis of the stack.
    #[rebuild(layout)]
    pub axis: Axis,
    /// How to justify the content along the main axis.
    #[rebuild(layout)]
    pub justify: Justify,
    /// How to align the content along the cross axis, within each line.
    #[rebuild(layout)]
    pub align: Align,
    /// The gap between children.
    #[rebuild(layout)]
    pub gap: f32,
}

impl<V> Stack<V> {
    /// Create a new [`Stack`].
    pub fn new(axis: Axis, content: V) -> Self {
        Self {
            content: PodSeq::new(content),
            axis,
            justify: Justify::Start,
            align: Align::Center,
            gap: 0.0,
        }
    }

    /// Create a new horizontal [`Stack`].
    pub fn hstack(content: V) -> Self {
        Self::new(Axis::Horizontal, content)
    }

    /// Create a new vertical [`Stack`].
    pub fn vstack(content: V) -> Self {
        Self::new(Axis::Vertical, content)
    }
}

impl<'a, T> Stack<Vec<Box<dyn AnyView<T> + 'a>>> {
    /// Create a new horizontal [`Stack`], with dynamic content.
    pub fn hstack_any() -> Self {
        Self::hstack(Vec::new())
    }

    /// Create a new vertical [`Stack`], with dynamic content.
    pub fn vstack_any() -> Self {
        Self::vstack(Vec::new())
    }

    /// Push a view to the stack.
    pub fn push(&mut self, view: impl AnyView<T> + 'a) {
        self.content.push(Box::new(view));
    }
}

impl<V> Stack<V> {
    fn event_first<T>(
        &mut self,
        content: &mut SeqState<T, V>,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) where
        V: ViewSeq<T>,
    {
        for i in 0..self.content.len() {
            if event.is_handled() {
                break;
            }

            self.content.event_nth(i, content, cx, data, event);
        }
    }

    fn event_last<T>(
        &mut self,
        content: &mut SeqState<T, V>,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) where
        V: ViewSeq<T>,
    {
        for i in (0..self.content.len()).rev() {
            if event.is_handled() {
                break;
            }

            self.content.event_nth(i, content, cx, data, event);
        }
    }

    fn focus_next<T>(
        &mut self,
        content: &mut SeqState<T, V>,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
        focused: &Cell<bool>,
    ) where
        V: ViewSeq<T>,
    {
        let mut next = None;

        for i in 0..self.content.len() {
            self.content.event_nth(i, content, cx, data, event);

            if event.is_handled() {
                return;
            }

            if focused.get() {
                next = Some(i + 1);
                break;
            }
        }

        if let Some(next) = next {
            for i in next..self.content.len() {
                let focused = Event::new(RequestFocus::First);
                self.content.event_nth(i, content, cx, data, &focused);

                if focused.is_handled() {
                    event.handle();
                    break;
                }
            }
        }
    }

    fn focus_prev<T>(
        &mut self,
        content: &mut SeqState<T, V>,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
        focused: &Cell<bool>,
    ) where
        V: ViewSeq<T>,
    {
        let mut prev = None;

        for i in (0..self.content.len()).rev() {
            self.content.event_nth(i, content, cx, data, event);

            if event.is_handled() {
                return;
            }

            if focused.get() {
                prev = i.checked_sub(1);
                break;
            }
        }

        if let Some(prev) = prev {
            for i in (0..=prev).rev() {
                let focused = Event::new(RequestFocus::Last);
                self.content.event_nth(i, content, cx, data, &focused);

                if focused.is_handled() {
                    event.handle();
                    break;
                }
            }
        }
    }

    fn handle_focus<T>(
        &mut self,
        content: &mut SeqState<T, V>,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool
    where
        V: ViewSeq<T>,
    {
        match event.get::<SwitchFocus>() {
            Some(SwitchFocus::Next(focused)) => {
                self.focus_next(content, cx, data, event, focused);
                return true;
            }
            Some(SwitchFocus::Prev(focused)) => {
                self.focus_prev(content, cx, data, event, focused);
                return true;
            }
            None => {}
        }

        match event.get::<RequestFocus>() {
            Some(RequestFocus::First) => {
                self.event_first(content, cx, data, event);
                return true;
            }
            Some(RequestFocus::Last) => {
                self.event_last(content, cx, data, event);
                return true;
            }
            None => {}
        }

        false
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct StackState {
    flex_sum: f32,
    majors: Vec<f32>,
    minors: Vec<f32>,
}

impl StackState {
    fn new(len: usize) -> Self {
        Self {
            flex_sum: 0.0,
            majors: vec![0.0; len],
            minors: vec![0.0; len],
        }
    }

    fn resize(&mut self, len: usize) {
        self.majors.resize(len, 0.0);
        self.minors.resize(len, 0.0);
    }

    fn major(&self) -> f32 {
        self.majors.iter().copied().sum()
    }

    fn minor(&self) -> f32 {
        let mut total = 0.0;

        for minor in self.minors.iter().copied() {
            total = f32::max(total, minor);
        }

        total
    }
}

impl<T, V: ViewSeq<T>> View<T> for Stack<V> {
    type State = (StackState, SeqState<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        (
            StackState::new(self.content.len()),
            self.content.build(cx, data),
        )
    }

    fn rebuild(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        Rebuild::rebuild(self, cx, old);

        if self.content.len() != old.content.len() {
            state.resize(self.content.len());
            cx.request_layout();
        }

        (self.content).rebuild(content, &mut cx.build_cx(), data, &old.content);

        for i in 0..self.content.len() {
            self.content.rebuild_nth(i, content, cx, data, &old.content)
        }
    }

    fn event(
        &mut self,
        (_, content): &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) {
        if self.handle_focus(content, cx, data, event) {
            return;
        }

        for i in 0..self.content.len() {
            self.content.event_nth(i, content, cx, data, event);
        }
    }

    fn layout(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        let (min_major, mut min_minor) = self.axis.unpack(space.min);
        let (max_major, max_minor) = self.axis.unpack(space.max);

        if self.align == Align::Stretch {
            min_minor = max_minor;
        }

        let total_gap = self.gap * (self.content.len() as f32 - 1.0);

        /* measure the non-flex content */

        state.flex_sum = 0.0;

        for i in 0..self.content.len() {
            if content[i].is_flex() {
                state.flex_sum += content[i].flex();
                state.majors[i] = 0.0;
                continue;
            }

            let space = Space::new(
                self.axis.pack(0.0, min_minor),
                self.axis.pack(f32::INFINITY, max_minor),
            );

            let size = self.content.layout_nth(i, content, cx, data, space);
            state.majors[i] = self.axis.major(size);
            state.minors[i] = self.axis.minor(size);

            if content[i].is_flex() {
                state.flex_sum += content[i].flex();
                cx.request_layout();
            }
        }

        /* measure the expanded content */

        let remaining = f32::max(max_major - total_gap - state.major(), 0.0);
        let per_flex = remaining / state.flex_sum;

        for i in 0..self.content.len() {
            if !content[i].is_flex() || content[i].is_tight() {
                continue;
            }

            let flex = content[i].flex();
            let major = per_flex * flex;

            let space = Space::new(
                self.axis.pack(0.0, min_minor),
                self.axis.pack(major, max_minor),
            );

            let size = self.content.layout_nth(i, content, cx, data, space);
            state.majors[i] = self.axis.major(size);
            state.minors[i] = self.axis.minor(size);
        }

        /* measure the flex content */

        let remaining = f32::max(max_major - total_gap - state.major(), 0.0);
        let per_flex = remaining / state.flex_sum;

        for i in 0..self.content.len() {
            if !content[i].is_flex() || !content[i].is_tight() {
                continue;
            }

            let flex = content[i].flex();
            let major = per_flex * flex;

            let space = Space::new(
                self.axis.pack(major, min_minor),
                self.axis.pack(major, max_minor),
            );

            let size = self.content.layout_nth(i, content, cx, data, space);
            state.majors[i] = self.axis.major(size);
            state.minors[i] = self.axis.minor(size);
        }

        /* position content */

        let major = f32::clamp(state.major() + total_gap, min_major, max_major);
        let minor = f32::clamp(state.minor(), min_minor, max_minor);

        for (i, child_major) in (self.justify)
            .layout(&state.majors, major, self.gap)
            .enumerate()
        {
            let child_minor = self.align.align(minor, state.minors[i]);
            let offset = self.axis.pack(child_major, child_minor);
            content[i].translate(offset);
        }

        self.axis.pack(major, minor)
    }

    fn draw(
        &mut self,
        (_, content): &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        for i in 0..self.content.len() {
            self.content.draw_nth(i, content, cx, data, canvas);
        }
    }
}
