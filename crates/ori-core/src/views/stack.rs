use std::ops::Deref;

use ori_macro::{example, Build, Styled};

use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Align, Axis, Justify, Size, Space},
    rebuild::Rebuild,
    style::{Styled, Styles},
    view::{AnyView, PodSeq, SeqState, View, ViewSeq},
};

pub use crate::{hstack, vstack};

use super::Flex;

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
pub fn hstack<V>(view: V) -> Stack<V> {
    Stack::horizontal(view)
}

/// Create a vertical [`Stack`].
pub fn vstack<V>(view: V) -> Stack<V> {
    Stack::vertical(view)
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
pub fn hstack_any<'a, T>() -> Stack<Vec<Box<dyn AnyView<T> + 'a>>> {
    Stack::any_horizontal()
}

/// Create a vertical [`Stack`], with dynamic content.
pub fn vstack_any<'a, T>() -> Stack<Vec<Box<dyn AnyView<T> + 'a>>> {
    Stack::any_vertical()
}

/// A view that stacks it's content in a line.
#[example(name = "stack", width = 400, height = 600)]
#[derive(Styled, Build, Rebuild)]
pub struct Stack<V> {
    /// The content of the stack.
    #[build(ignore)]
    pub content: PodSeq<V>,

    /// The axis of the stack.
    #[rebuild(layout)]
    pub axis: Axis,

    /// How to justify the content along the main axis.
    #[rebuild(layout)]
    #[styled(default)]
    pub justify: Styled<Justify>,

    /// How to align the content along the cross axis, within each line.
    #[rebuild(layout)]
    #[styled(default = Align::Center)]
    pub align: Styled<Align>,

    /// The gap between children.
    #[rebuild(layout)]
    #[styled(default)]
    pub gap: Styled<f32>,
}

impl<V> Stack<V> {
    /// Create a new [`Stack`].
    pub fn new(axis: Axis, content: V) -> Self {
        Self {
            content: PodSeq::new(content),
            axis,
            justify: Styled::style("stack.justify"),
            align: Styled::style("stack.align"),
            gap: Styled::style("stack.gap"),
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

impl<T> Stack<Vec<Box<dyn AnyView<T> + '_>>> {
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
pub struct StackState {
    style: StackStyle,
    flex_sum: f32,
    majors: Vec<f32>,
    minors: Vec<f32>,
}

impl StackState {
    fn new<T, V: ViewSeq<T>>(stack: &Stack<V>, styles: &Styles) -> Self {
        Self {
            style: StackStyle::styled(stack, styles),
            flex_sum: 0.0,
            majors: vec![0.0; stack.content.len()],
            minors: vec![0.0; stack.content.len()],
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
        cx.set_class("stack");

        (
            StackState::new(self, cx.styles()),
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
        state.style.rebuild(self, cx);

        if self.content.len() != old.content.len() {
            state.resize(self.content.len());
            cx.layout();
        }

        (self.content).rebuild(content, &mut cx.as_build_cx(), data, &old.content);

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
    ) -> bool {
        self.content.event(content, cx, data, event)
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

        // this avoids a panic in later clamp calls
        let min_major = min_major.min(max_major);
        let min_minor = min_minor.min(max_minor);

        let total_gap = state.style.gap * (self.content.len() as f32 - 1.0);

        /* measure the content */

        let stretch_full = state.style.align == Align::Stretch && min_minor == max_minor;

        if state.style.align == Align::Fill || stretch_full {
            layout(
                self, cx, content, state, data, max_major, max_minor, max_minor, total_gap,
            );
        } else {
            layout(
                self, cx, content, state, data, max_major, 0.0, max_minor, total_gap,
            );

            /* stretch the content */

            if state.style.align == Align::Stretch {
                let minor = f32::clamp(state.minor(), min_minor, max_minor);
                layout(
                    self, cx, content, state, data, max_major, minor, minor, total_gap,
                );
            }
        }

        /* position content */

        let major = f32::clamp(state.major() + total_gap, min_major, max_major);
        let minor = f32::clamp(state.minor(), min_minor, max_minor);

        for (i, child_major) in (state.style.justify)
            .layout(&state.majors, major, state.style.gap)
            .enumerate()
        {
            let child_align = state.style.align.align(minor, state.minors[i]);
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

#[allow(clippy::too_many_arguments)]
fn layout<T, V: ViewSeq<T>>(
    stack: &mut Stack<V>,
    cx: &mut LayoutCx,
    content: &mut SeqState<T, V>,
    state: &mut StackState,
    data: &mut T,
    max_major: f32,
    min_minor: f32,
    max_minor: f32,
    total_gap: f32,
) {
    state.flex_sum = 0.0;

    /* measure the non-flex content */

    for i in 0..stack.content.len() {
        if let Some(flex) = content[i].get_property::<Flex>() {
            state.flex_sum += flex.amount;
            state.majors[i] = 0.0;
            continue;
        }

        let space = Space::new(
            stack.axis.pack(0.0, min_minor),
            stack.axis.pack(f32::INFINITY, max_minor),
        );

        let size = stack.content.layout_nth(i, content, cx, data, space);
        state.majors[i] = stack.axis.major(size);
        state.minors[i] = stack.axis.minor(size);
    }

    /* measure the expanded content */

    let remaining = f32::max(max_major - total_gap - state.major(), 0.0);
    let per_flex = remaining / state.flex_sum;

    for i in 0..stack.content.len() {
        let Some(flex) = content[i].get_property::<Flex>() else {
            continue;
        };

        if !flex.is_tight {
            continue;
        }

        let major = per_flex * flex.amount;

        let space = Space::new(
            stack.axis.pack(0.0, min_minor),
            stack.axis.pack(major, max_minor),
        );

        let size = stack.content.layout_nth(i, content, cx, data, space);
        state.majors[i] = stack.axis.major(size);
        state.minors[i] = stack.axis.minor(size);
    }

    /* measure the flex content */

    let remaining = f32::max(max_major - total_gap - state.major(), 0.0);
    let per_flex = remaining / state.flex_sum;

    for i in 0..stack.content.len() {
        let Some(flex) = content[i].get_property::<Flex>() else {
            continue;
        };

        if flex.is_tight {
            continue;
        }

        let major = per_flex * flex.amount;

        let space = Space::new(
            stack.axis.pack(major, min_minor),
            stack.axis.pack(major, max_minor),
        );

        let size = stack.content.layout_nth(i, content, cx, data, space);
        state.majors[i] = stack.axis.major(size);
        state.minors[i] = stack.axis.minor(size);
    }
}
