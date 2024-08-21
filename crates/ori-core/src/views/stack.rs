use std::ops::Deref;

use ori_macro::{example, Build};

use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Align, Axis, Justify, Size, Space},
    rebuild::Rebuild,
    view::{AnyView, PodSeq, SeqState, View, ViewSeq},
};

pub use crate::{hstack, vstack};

/// Create a horizontal [`Stack`].
#[macro_export]
macro_rules! hstack {
    ($($child:expr),* $(,)?) => {
        $crate::views::hstack(($($child,)*))
    };
}

/// Create a vertical [`Stack`].
#[macro_export]
macro_rules! vstack {
    ($($child:expr),* $(,)?) => {
        $crate::views::vstack(($($child,)*))
    };
}

/// Create a horizontal [`Stack`].
pub fn hstack<V>(content: V) -> Stack<V> {
    Stack::horizontal(content)
}

/// Create a vertical [`Stack`].
pub fn vstack<V>(content: V) -> Stack<V> {
    Stack::vertical(content)
}

/// Create a horizontal [`Stack`], with vector content.
pub fn hstack_vec<V>() -> Stack<Vec<V>> {
    Stack::vec_horizontal()
}

/// Create a vertical [`Stack`], with vector content.
pub fn vstack_vec<V>() -> Stack<Vec<V>> {
    Stack::vec_vertical()
}

/// Create a horizontal [`Stack`], with dynamic content.
pub fn hstack_any<'a, V>() -> Stack<Vec<Box<dyn AnyView<V> + 'a>>> {
    Stack::any_horizontal()
}

/// Create a vertical [`Stack`], with dynamic content.
pub fn vstack_any<'a, V>() -> Stack<Vec<Box<dyn AnyView<V> + 'a>>> {
    Stack::any_vertical()
}

/// A view that stacks it's content in a line.
#[example(name = "stack", width = 400, height = 600)]
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
    pub fn horizontal(content: V) -> Self {
        Self::new(Axis::Horizontal, content)
    }

    /// Create a new vertical [`Stack`].
    pub fn vertical(content: V) -> Self {
        Self::new(Axis::Vertical, content)
    }
}

impl<T> Stack<Vec<T>> {
    /// Create a new [`Stack`], with vector content.
    pub fn vec(axis: Axis) -> Self {
        Self::new(axis, Vec::new())
    }

    /// Create a new horizontal [`Stack`], with vector content.
    pub fn vec_horizontal() -> Self {
        Self::horizontal(Vec::new())
    }

    /// Create a new vertical [`Stack`], with vector content.
    pub fn vec_vertical() -> Self {
        Self::vertical(Vec::new())
    }

    /// Push a view to the stack.
    pub fn push(&mut self, view: T) {
        self.content.push(view);
    }

    /// Push a view to the stack.
    pub fn with(mut self, view: T) -> Self {
        self.push(view);
        self
    }

    /// Get whether the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.content.deref().is_empty()
    }

    /// Get the number of views in the stack.
    pub fn len(&self) -> usize {
        self.content.deref().len()
    }
}

impl<'a, T> Stack<Vec<Box<dyn AnyView<T> + 'a>>> {
    /// Create a new [`Stack`], with dynamic content.
    pub fn any(axis: Axis) -> Self {
        Self::new(axis, Vec::new())
    }

    /// Create a new horizontal [`Stack`], with dynamic content.
    pub fn any_horizontal() -> Self {
        Self::horizontal(Vec::new())
    }

    /// Create a new vertical [`Stack`], with dynamic content.
    pub fn any_vertical() -> Self {
        Self::vertical(Vec::new())
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
        let (min_major, min_minor) = self.axis.unpack(space.min);
        let (max_major, max_minor) = self.axis.unpack(space.max);

        let total_gap = self.gap * (self.content.len() as f32 - 1.0);

        /* measure the non-flex content */

        state.flex_sum = 0.0;

        for i in 0..self.content.len() {
            if content[i].is_flex() {
                state.flex_sum += content[i].flex();
                state.majors[i] = 0.0;
                continue;
            }

            let space = Space::new(Size::ZERO, self.axis.pack(f32::INFINITY, max_minor));

            let size = self.content.layout_nth(i, content, cx, data, space);
            state.majors[i] = self.axis.major(size);
            state.minors[i] = self.axis.minor(size);
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

            let space = Space::new(Size::ZERO, self.axis.pack(major, max_minor));

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

            let space = Space::new(self.axis.pack(major, 0.0), self.axis.pack(major, max_minor));

            let size = self.content.layout_nth(i, content, cx, data, space);
            state.majors[i] = self.axis.major(size);
            state.minors[i] = self.axis.minor(size);
        }

        /* stretch the content */

        if self.align.is_stretch() {
            let minor = state.minor();
            let minor = f32::clamp(minor, min_minor, max_minor);

            state.flex_sum = 0.0;

            /* re-measure the non-flex content */

            for i in 0..self.content.len() {
                if content[i].is_flex() {
                    state.flex_sum += content[i].flex();
                    state.majors[i] = 0.0;
                    continue;
                }

                let space = Space {
                    min: self.axis.pack(0.0, minor),
                    max: self.axis.pack(f32::INFINITY, minor),
                };

                // calling layout_nth again is not ideal, as it can lead to exponential time complexity
                // but considering how cheap layout generally is, this *should* be fine
                let size = self.content.layout_nth(i, content, cx, data, space);

                state.majors[i] = self.axis.major(size);
                state.minors[i] = self.axis.minor(size);
            }

            /* re-measure the expanded content */

            let remaining = f32::max(max_major - total_gap - state.major(), 0.0);
            let per_flex = remaining / state.flex_sum;

            for i in 0..self.content.len() {
                if !content[i].is_flex() || content[i].is_tight() {
                    continue;
                }

                let flex = content[i].flex();
                let major = per_flex * flex;

                let space = Space {
                    min: self.axis.pack(0.0, minor),
                    max: self.axis.pack(major, minor),
                };

                let size = self.content.layout_nth(i, content, cx, data, space);

                state.majors[i] = self.axis.major(size);
                state.minors[i] = self.axis.minor(size);
            }

            /* re-measure the flex content */

            let remaining = f32::max(max_major - total_gap - state.major(), 0.0);
            let per_flex = remaining / state.flex_sum;

            for i in 0..self.content.len() {
                if !content[i].is_flex() || !content[i].is_tight() {
                    continue;
                }

                let flex = content[i].flex();
                let major = per_flex * flex;

                let space = Space {
                    min: self.axis.pack(major, minor),
                    max: self.axis.pack(major, minor),
                };

                let size = self.content.layout_nth(i, content, cx, data, space);

                state.majors[i] = self.axis.major(size);
                state.minors[i] = self.axis.minor(size);
            }
        }

        /* position content */

        let major = f32::clamp(state.major() + total_gap, min_major, max_major);
        let minor = f32::clamp(state.minor(), min_minor, max_minor);

        for (i, child_major) in (self.justify)
            .layout(&state.majors, major, self.gap)
            .enumerate()
        {
            let child_align = self.align.align(minor, state.minors[i]);
            let offset = self.axis.pack(child_major, child_align);
            content[i].translate(offset);
        }

        self.axis.pack(major, minor)
    }

    fn draw(&mut self, (_, content): &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        for i in 0..self.content.len() {
            self.content.draw_nth(i, content, cx, data);
        }
    }
}
