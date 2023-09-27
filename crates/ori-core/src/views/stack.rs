use std::cell::Cell;

use crate::{
    canvas::Canvas,
    event::{Event, RequestFocus, SwitchFocus},
    layout::{Align, Axis, Justify, Size, Space},
    log::warn_internal,
    rebuild::Rebuild,
    view::{BuildCx, DrawCx, EventCx, LayoutCx, PodSeq, RebuildCx, SeqState, View, ViewSeq},
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

/// A view that stacks its content in a line.
#[derive(Rebuild)]
pub struct Stack<V> {
    /// The content of the stack.
    pub content: PodSeq<V>,
    /// The size of the stack.
    #[rebuild(layout)]
    pub space: Space,
    /// The axis of the stack.
    #[rebuild(layout)]
    pub axis: Axis,
    /// Whether the stack should wrap its content.
    #[rebuild(layout)]
    pub wrap: bool,
    /// How to justify the content along the main axis.
    #[rebuild(layout)]
    pub justify_content: Justify,
    /// How to align the content along the cross axis, within each line.
    #[rebuild(layout)]
    pub align_items: Align,
    /// How to align the lines along the cross axis.
    #[rebuild(layout)]
    pub align_content: Justify,
    /// The gap between columns.
    #[rebuild(layout)]
    pub column_gap: f32,
    /// The gap between rows.
    #[rebuild(layout)]
    pub row_gap: f32,
}

impl<V> Stack<V> {
    /// Create a new [`Stack`].
    pub fn new(axis: Axis, content: V) -> Self {
        Self {
            content: PodSeq::new(content),
            space: Space::UNBOUNDED,
            axis,
            wrap: false,
            justify_content: Justify::Start,
            align_items: Align::Center,
            align_content: Justify::Start,
            column_gap: 0.0,
            row_gap: 0.0,
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

    /// Set the space of the stack.
    pub fn space(mut self, space: impl Into<Space>) -> Self {
        self.space = space.into();
        self
    }

    /// Set the size of the stack.
    pub fn size(mut self, size: impl Into<Size>) -> Self {
        self.space = Space::from_size(size.into());
        self
    }

    /// Set the width of the stack.
    pub fn width(mut self, width: f32) -> Self {
        self.space.min.width = width;
        self.space.max.width = width;
        self
    }

    /// Set the height of the stack.
    pub fn height(mut self, height: f32) -> Self {
        self.space.min.height = height;
        self.space.max.height = height;
        self
    }

    /// Set the minimum width of the stack.
    pub fn min_width(mut self, min_width: f32) -> Self {
        self.space.min.width = min_width;
        self
    }

    /// Set the minimum height of the stack.
    pub fn min_height(mut self, min_height: f32) -> Self {
        self.space.min.height = min_height;
        self
    }

    /// Set the maximum width of the stack.
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.space.max.width = max_width;
        self
    }

    /// Set the maximum height of the stack.
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.space.max.height = max_height;
        self
    }

    /// Set the axis of the stack.
    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    /// Set whether the stack should wrap its content.
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Set the justify content.
    pub fn justify_content(mut self, justify: impl Into<Justify>) -> Self {
        self.justify_content = justify.into();
        self
    }

    /// Set the align items.
    pub fn align_items(mut self, align: impl Into<Align>) -> Self {
        self.align_items = align.into();
        self
    }

    /// Set the align content.
    pub fn align_content(mut self, align: impl Into<Justify>) -> Self {
        self.align_content = align.into();
        self
    }

    /// Set the gap between columns and rows.
    pub fn gap(mut self, gap: f32) -> Self {
        self.column_gap = gap;
        self.row_gap = gap;
        self
    }

    /// Set the gap between columns.
    pub fn column_gap(mut self, gap: f32) -> Self {
        self.column_gap = gap;
        self
    }

    /// Set the gap between rows.
    pub fn row_gap(mut self, gap: f32) -> Self {
        self.row_gap = gap;
        self
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

    #[allow(clippy::too_many_arguments)]
    fn measure_fixed<T>(
        &mut self,
        state: &mut StackState,
        content: &mut SeqState<T, V>,
        data: &mut T,
        cx: &mut LayoutCx,
        gap_major: f32,
        max_major: f32,
        max_minor: f32,
    ) where
        V: ViewSeq<T>,
    {
        state.lines.clear();

        let mut major = 0.0;
        let mut minor = 0.0f32;
        let mut flex_grow_sum = 0.0;
        let mut flex_shrink_sum = 0.0;

        let mut start = 0;

        for i in 0..self.content.len() {
            let content_space = if self.wrap {
                Space::UNBOUNDED
            } else {
                Space::new(Size::ZERO, self.axis.pack(f32::INFINITY, max_minor))
            };

            let mut size = (self.content).layout_nth(i, content, cx, data, content_space);

            if content[i].is_flex() {
                size = size.min(self.axis.pack(max_major, max_minor));
            } else if !size.is_finite() && max_major.is_finite() && max_minor.is_finite() {
                warn_internal!(
                    "A non-flex view in a stack has an infinite size, [{}] = {}",
                    i,
                    size,
                );
            }

            let (child_major, child_minor) = self.axis.unpack(size);
            state.majors[i] = child_major;
            state.minors[i] = child_minor;

            let gap = if i > 0 { gap_major } else { 0.0 };

            if self.wrap && major + child_major + gap > max_major {
                state.lines.push(StackLine {
                    start,
                    end: i,
                    major,
                    minor: minor.min(max_minor),
                    flex_grow_sum,
                    flex_shrink_sum,
                });

                start = i;
                major = child_major;
                minor = child_minor;
                flex_grow_sum = content[i].flex_grow;
                flex_shrink_sum = content[i].flex_shrink;
            } else {
                major += child_major + gap;
                minor = minor.max(child_minor);
                flex_grow_sum += content[i].flex_grow;
                flex_shrink_sum += content[i].flex_shrink;
            }
        }

        state.lines.push(StackLine {
            start,
            end: self.content.len(),
            major,
            minor: minor.min(max_minor),
            flex_grow_sum,
            flex_shrink_sum,
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn measure_flex<T>(
        &mut self,
        state: &mut StackState,
        content: &mut SeqState<T, V>,
        data: &mut T,
        cx: &mut LayoutCx,
        min_major: f32,
        max_major: f32,
        max_minor: f32,
    ) where
        V: ViewSeq<T>,
    {
        for line in state.lines.iter_mut() {
            let overflow = line.major - max_major;
            let underflow = min_major - line.major;

            let px_per_flex = if overflow > 0.0 {
                -overflow / line.flex_shrink_sum
            } else if underflow > 0.0 {
                underflow / line.flex_grow_sum
            } else {
                0.0
            };

            for i in line.start..line.end {
                let flex = if overflow > 0.0 {
                    content[i].flex_shrink
                } else {
                    content[i].flex_grow
                };

                if flex == 0.0 && !self.align_items.is_stretch() {
                    continue;
                }

                let desired_major = state.majors[i] + px_per_flex * flex;

                let min_major = if content[i].is_grow() {
                    desired_major
                } else {
                    0.0
                };

                let max_major = if content[i].is_shrink() {
                    desired_major
                } else {
                    f32::INFINITY
                };

                let space = if self.align_items.is_stretch() {
                    Space::new(
                        self.axis.pack(min_major, line.minor),
                        self.axis.pack(max_major, line.minor),
                    )
                } else {
                    Space::new(
                        self.axis.pack(min_major, 0.0),
                        self.axis.pack(max_major, max_minor),
                    )
                };

                let size = self.content.layout_nth(i, content, cx, data, space);
                let (child_major, child_minor) = self.axis.unpack(size);

                line.major += child_major - state.majors[i];
                line.minor = line.minor.max(child_minor);
                state.majors[i] = child_major;
                state.minors[i] = child_minor;
            }
        }
    }
}

#[derive(Debug)]
struct StackLine {
    start: usize,
    end: usize,
    major: f32,
    minor: f32,
    flex_grow_sum: f32,
    flex_shrink_sum: f32,
}

#[doc(hidden)]
#[derive(Debug)]
pub struct StackState {
    lines: Vec<StackLine>,
    line_offsets: Vec<f32>,
    majors: Vec<f32>,
    minors: Vec<f32>,
}

impl StackState {
    fn new(len: usize) -> Self {
        Self {
            lines: Vec::new(),
            line_offsets: Vec::new(),
            majors: vec![0.0; len],
            minors: vec![0.0; len],
        }
    }

    fn resize(&mut self, len: usize) {
        self.majors.resize(len, 0.0);
        self.minors.resize(len, 0.0);
    }

    fn major(&self) -> f32 {
        let mut major = 0.0f32;

        for line in self.lines.iter() {
            major = major.max(line.major);
        }

        major
    }

    fn minor(&self, minor_gap: f32) -> f32 {
        let total_gap = minor_gap * (self.lines.len() - 1) as f32;
        self.lines.iter().map(|line| line.minor).sum::<f32>() + total_gap
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
        let space = self.space.constrain(space);

        let (max_major, max_minor) = self.axis.unpack(space.max);
        let (min_major, min_minor) = self.axis.unpack(space.min);

        let (gap_major, gap_minor) = self.axis.unpack((self.column_gap, self.row_gap));

        self.measure_fixed(state, content, data, cx, gap_major, max_major, max_minor);
        self.measure_flex(state, content, data, cx, min_major, max_major, max_minor);

        if !self.wrap {
            state.lines[0].minor = state.lines[0].minor.clamp(min_minor, max_minor);
        }

        let content_major = state.major().min(max_major);
        let content_minor = state.minor(gap_minor).max(min_minor);

        let mut size = self.axis.pack(content_major, content_minor);
        size = Size::max(size, space.min);

        let (major, minor) = self.axis.unpack(size);

        state.line_offsets.resize(state.lines.len(), 0.0);

        self.align_content.layout(
            state.lines.iter().map(|line| line.minor),
            |index, offset| state.line_offsets[index] = offset,
            minor,
            gap_minor,
        );

        for (i, line) in state.lines.iter().enumerate() {
            let line_offset = state.line_offsets[i];
            let child_majors = &state.majors[line.start..line.end];
            let child_minors = &state.minors[line.start..line.end];

            self.justify_content.layout(
                child_majors.iter().copied(),
                |index, offset| {
                    let align = self.align_items.align(line.minor, child_minors[index]);
                    let offset = self.axis.pack(offset, line_offset + align);
                    content[line.start + index].translate(offset);
                },
                major,
                gap_major,
            );
        }

        size
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
