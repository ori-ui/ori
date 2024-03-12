use std::ops::Range;

use ori_macro::Build;

use crate::{
    canvas::Canvas,
    event::Event,
    layout::{Align, Axis, Justify, Size, Space},
    rebuild::Rebuild,
    view::{
        AnyView, BuildCx, DrawCx, EventCx, LayoutCx, PodSeq, RebuildCx, SeqState, View, ViewSeq,
    },
};

pub use crate::{hwrap, vwrap};

/// Create a horizontal [`Wrap`].
#[macro_export]
macro_rules! hwrap {
    ($($child:expr),* $(,)?) => {
        $crate::views::hwrap(($($child,)*))
    };
}

/// Create a vertical [`Wrap`].
#[macro_export]
macro_rules! vwrap {
    ($($child:expr),* $(,)?) => {
        $crate::views::vwrap(($($child,)*))
    };
}

/// Create a horizontal [`Wrap`].
pub fn hwrap<V>(content: V) -> Wrap<V> {
    Wrap::new(Axis::Horizontal, content)
}

/// Create a vertical [`Wrap`].
pub fn vwrap<V>(content: V) -> Wrap<V> {
    Wrap::new(Axis::Vertical, content)
}

/// Create a horizontal [`Wrap`], with dynamic content.
pub fn hwrap_any<'a, V>() -> Wrap<Vec<Box<dyn AnyView<V> + 'a>>> {
    Wrap::hwrap_any()
}

/// Create a vertical [`Wrap`], with dynamic content.
pub fn vwrap_any<'a, V>() -> Wrap<Vec<Box<dyn AnyView<V> + 'a>>> {
    Wrap::vwrap_any()
}

/// A view that lays out it's content in a line wrapping if it doesn't fit.
///
/// Note that unlike [`Stack`](super::Stack) this view does not care about flex.
#[derive(Build, Rebuild)]
pub struct Wrap<V> {
    /// The content.
    #[build(ignore)]
    pub content: PodSeq<V>,
    /// The axis.
    #[rebuild(layout)]
    pub axis: Axis,
    /// How to justify the content along the main axis.
    #[rebuild(layout)]
    pub justify: Justify,
    /// How to align the content along the cross axis.
    #[rebuild(layout)]
    pub align: Align,
    /// How to justify the content along the cross axis.
    #[rebuild(layout)]
    pub justify_cross: Justify,
    /// The gap between each row.
    #[rebuild(layout)]
    pub row_gap: f32,
    /// The gap between each column.
    #[rebuild(layout)]
    pub column_gap: f32,
}

impl<V> Wrap<V> {
    /// Create a new [`Wrap`].
    pub fn new(axis: Axis, content: V) -> Self {
        Self {
            content: PodSeq::new(content),
            axis,
            justify: Justify::Start,
            align: Align::Center,
            justify_cross: Justify::Start,
            row_gap: 0.0,
            column_gap: 0.0,
        }
    }

    /// Create a new horizontal [`Wrap`].
    pub fn hwrap(content: V) -> Self {
        Self::new(Axis::Horizontal, content)
    }

    /// Create a new vertical [`Wrap`].
    pub fn vwrap(content: V) -> Self {
        Self::new(Axis::Vertical, content)
    }

    /// Set the gap for both the rows and columns.
    pub fn gap(mut self, gap: f32) -> Self {
        self.row_gap = gap;
        self.column_gap = gap;
        self
    }
}

impl<'a, T> Wrap<Vec<Box<dyn AnyView<T> + 'a>>> {
    /// Create a new horizontal [`Wrap`], with dynamic content.
    pub fn hwrap_any() -> Self {
        Self::hwrap(Vec::new())
    }

    /// Create a new vertical [`Wrap`], with dynamic content.
    pub fn vwrap_any() -> Self {
        Self::vwrap(Vec::new())
    }

    /// Push a view to the stack.
    pub fn push(&mut self, view: impl AnyView<T> + 'a) {
        self.content.push(Box::new(view));
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct WrapState {
    majors: Vec<f32>,
    runs: Vec<Range<usize>>,
    run_minors: Vec<f32>,
}

impl WrapState {
    fn new(len: usize) -> Self {
        Self {
            majors: vec![0.0; len],
            runs: Vec::new(),
            run_minors: Vec::new(),
        }
    }

    fn resize(&mut self, len: usize) {
        self.majors.resize(len, 0.0);
    }

    fn minor(&self) -> f32 {
        self.run_minors.iter().copied().sum()
    }
}

impl<T, V: ViewSeq<T>> View<T> for Wrap<V> {
    type State = (WrapState, SeqState<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        (
            WrapState::new(self.content.len()),
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
            self.content.rebuild_nth(i, content, cx, data, &old.content);
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

        for i in 0..self.content.len() {
            let size = (self.content).layout_nth(i, content, cx, data, Space::UNBOUNDED);
            state.majors[i] = self.axis.major(size);
        }

        let (major_gap, minor_gap) = self.axis.unpack((self.row_gap, self.column_gap));

        let mut major = 0.0;

        state.runs.clear();
        state.run_minors.clear();

        let mut run_start = 0;
        let mut run_major = 0.0;
        let mut run_minor = 0.0;

        for i in 0..self.content.len() {
            let (child_major, child_minor) = self.axis.unpack(content[i].size());
            let gap = if run_major > 0.0 { major_gap } else { 0.0 };

            if run_major + gap <= max_major {
                run_major += gap + child_major;
                run_minor = f32::max(run_minor, child_minor);
                continue;
            }

            state.runs.push(run_start..i);
            state.run_minors.push(run_minor);
            major = f32::max(major, run_major);

            run_start = i;
            run_major = child_major;
            run_minor = child_minor;
        }

        state.runs.push(run_start..self.content.len());
        state.run_minors.push(run_minor);
        major = f32::max(major, run_major);

        let total_minor_gap = minor_gap * (state.runs.len() as f32 - 1.0);

        let major = f32::clamp(major, min_major, max_major);
        let minor = f32::clamp(state.minor() + total_minor_gap, min_minor, max_minor);

        for (i, run_position) in (self.justify_cross)
            .layout(&state.run_minors, minor, minor_gap)
            .enumerate()
        {
            let run = state.runs[i].clone();
            let run_minor = state.run_minors[i];

            for (child_position, j) in (self.justify)
                .layout(&state.majors[run.clone()], major, major_gap)
                .zip(run)
            {
                let child_minor = self.axis.minor(content[j].size());
                let child_align = self.align.align(run_minor, child_minor);
                let offset = self.axis.pack(child_position, run_position + child_align);
                content[j].translate(offset);
            }
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
