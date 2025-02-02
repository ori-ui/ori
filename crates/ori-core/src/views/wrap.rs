use std::ops::{Deref, Range};

use ori_macro::{example, Build};

use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Align, Axis, Justify, Size, Space},
    rebuild::Rebuild,
    style::{Stylable, Styled, Styles},
    view::{AnyView, PodSeq, SeqState, View, ViewSeq},
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
pub fn hwrap<V>(view: V) -> Wrap<V> {
    Wrap::new(Axis::Horizontal, view)
}

/// Create a vertical [`Wrap`].
pub fn vwrap<V>(view: V) -> Wrap<V> {
    Wrap::new(Axis::Vertical, view)
}

/// Create a horizontal [`Wrap`], with a vector of content.
pub fn hwrap_vec<V>() -> Wrap<Vec<V>> {
    Wrap::horizontal_vec()
}

/// Create a vertical [`Wrap`], with a vector of content.
pub fn vwrap_vec<V>() -> Wrap<Vec<V>> {
    Wrap::vertical_vec()
}

/// Create a horizontal [`Wrap`], with dynamic content.
pub fn hwrap_any<'a, T>() -> Wrap<Vec<Box<dyn AnyView<T> + 'a>>> {
    Wrap::horizontal_any()
}

/// Create a vertical [`Wrap`], with dynamic content.
pub fn vwrap_any<'a, T>() -> Wrap<Vec<Box<dyn AnyView<T> + 'a>>> {
    Wrap::vertical_any()
}

/// A view that lays out it's content in a line wrapping if it doesn't fit.
///
/// Note that unlike [`Stack`](super::Stack) this view does not care about flex.
#[example(name = "wrap", width = 400, height = 600)]
#[derive(Stylable, Build, Rebuild)]
pub struct Wrap<V> {
    /// The content.
    #[build(ignore)]
    pub content: PodSeq<V>,

    /// The axis.
    #[rebuild(layout)]
    pub axis: Axis,

    /// How to justify the content along the main axis.
    #[rebuild(layout)]
    #[style(default)]
    pub justify: Styled<Justify>,

    /// How to align the content along the cross axis.
    #[rebuild(layout)]
    #[style(default = Align::Center)]
    pub align: Styled<Align>,

    /// How to justify the content along the cross axis.
    #[rebuild(layout)]
    #[style(default)]
    pub justify_cross: Styled<Justify>,

    /// The gap between each row.
    #[rebuild(layout)]
    #[style(default)]
    pub row_gap: Styled<f32>,

    /// The gap between each column.
    #[rebuild(layout)]
    #[style(default)]
    pub column_gap: Styled<f32>,
}

impl<V> Wrap<V> {
    /// Create a new [`Wrap`].
    pub fn new(axis: Axis, content: V) -> Self {
        Self {
            content: PodSeq::new(content),
            axis,
            justify: Styled::style("wrap.justify"),
            align: Styled::style("wrap.align"),
            justify_cross: Styled::style("wrap.justify-cross"),
            row_gap: Styled::style("wrap.row-gap"),
            column_gap: Styled::style("wrap.column-gap"),
        }
    }

    /// Create a new horizontal [`Wrap`].
    pub fn horizontal(content: V) -> Self {
        Self::new(Axis::Horizontal, content)
    }

    /// Create a new vertical [`Wrap`].
    pub fn vertical(content: V) -> Self {
        Self::new(Axis::Vertical, content)
    }

    /// Set the gap for both the rows and columns.
    pub fn gap(mut self, gap: impl Into<Styled<f32>>) -> Self {
        self.row_gap = gap.into();
        self.column_gap = self.row_gap.clone();
        self
    }
}

impl<T> Wrap<Vec<T>> {
    /// Create a new [`Wrap`], with a vector of content.
    pub fn vec(axis: Axis) -> Self {
        Self::new(axis, Vec::new())
    }

    /// Create a new horizontal [`Wrap`], with a vector of content.
    pub fn horizontal_vec() -> Self {
        Self::horizontal(Vec::new())
    }

    /// Create a new vertical [`Wrap`], with a vector of content.
    pub fn vertical_vec() -> Self {
        Self::vertical(Vec::new())
    }

    /// Push a view to the wrap.
    pub fn push(&mut self, view: T) {
        self.content.push(view);
    }

    /// Push a view to the wrap.
    pub fn with(mut self, view: T) -> Self {
        self.push(view);
        self
    }

    /// Get whether the wrap is empty.
    pub fn is_empty(&self) -> bool {
        self.content.deref().is_empty()
    }

    /// Get the number of views in the wrap.
    pub fn len(&self) -> usize {
        self.content.deref().len()
    }
}

impl<T> Wrap<Vec<Box<dyn AnyView<T> + '_>>> {
    /// Create a new [`Wrap`], with dynamic content.
    pub fn any(axis: Axis) -> Self {
        Self::new(axis, Vec::new())
    }

    /// Create a new horizontal [`Wrap`], with dynamic content.
    pub fn horizontal_any() -> Self {
        Self::horizontal(Vec::new())
    }

    /// Create a new vertical [`Wrap`], with dynamic content.
    pub fn vertical_any() -> Self {
        Self::vertical(Vec::new())
    }
}

#[doc(hidden)]
pub struct WrapState<V> {
    style: WrapStyle<V>,
    majors: Vec<f32>,
    runs: Vec<Range<usize>>,
    run_minors: Vec<f32>,
}

impl<V> WrapState<V> {
    fn new<T>(wrap: &Wrap<V>, styles: &Styles) -> Self
    where
        V: ViewSeq<T>,
    {
        Self {
            style: wrap.style(styles),
            majors: vec![0.0; wrap.content.len()],
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
    type State = (WrapState<V>, SeqState<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        cx.set_class("wrap");

        (
            WrapState::new(self, cx.styles()),
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
            self.content.rebuild_nth(i, content, cx, data, &old.content);
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

        for i in 0..self.content.len() {
            let size = (self.content).layout_nth(i, content, cx, data, Space::UNBOUNDED);
            state.majors[i] = self.axis.major(size);
        }

        let gaps = (state.style.row_gap, state.style.column_gap);
        let (major_gap, minor_gap) = self.axis.unpack(gaps);

        let mut major = 0.0;

        state.runs.clear();
        state.run_minors.clear();

        let mut run_start = 0;
        let mut run_major = 0.0;
        let mut run_minor = 0.0;

        for i in 0..self.content.len() {
            let (child_major, child_minor) = self.axis.unpack(content[i].size());
            let gap = if run_major > 0.0 { major_gap } else { 0.0 };

            if run_major + child_major + gap <= max_major {
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

        for (i, run_position) in (state.style.justify_cross)
            .layout(&state.run_minors, minor, minor_gap)
            .enumerate()
        {
            let run = state.runs[i].clone();
            let run_minor = state.run_minors[i];

            for (child_position, j) in (state.style.justify)
                .layout(&state.majors[run.clone()], major, major_gap)
                .zip(run)
            {
                let child_minor = self.axis.minor(content[j].size());
                let child_align = state.style.align.align(run_minor, child_minor);
                let offset = self.axis.pack(child_position, run_position + child_align);
                content[j].translate(offset);
            }
        }

        self.axis.pack(major, minor)
    }

    fn draw(&mut self, (_, content): &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        for i in 0..self.content.len() {
            self.content.draw_nth(i, content, cx, data);
        }
    }
}
