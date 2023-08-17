use crate::{
    AlignItems, Axis, BuildCx, Canvas, DrawCx, Event, EventCx, Justify, LayoutCx, PodSequence,
    PodSequenceState, Rebuild, RebuildCx, Size, Space, View, ViewSequence,
};

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

/// Create a horizontal stack.
pub fn hstack<T, V: ViewSequence<T>>(content: V) -> Stack<T, V> {
    Stack::hstack(content)
}

/// Create a vertical stack.
pub fn vstack<T, V: ViewSequence<T>>(content: V) -> Stack<T, V> {
    Stack::vstack(content)
}

#[derive(Rebuild)]
pub struct Stack<T, V> {
    pub content: PodSequence<T, V>,
    #[rebuild(layout)]
    pub size: Option<Size>,
    #[rebuild(layout)]
    pub axis: Axis,
    #[rebuild(layout)]
    pub justify_content: Justify,
    #[rebuild(layout)]
    pub align_items: AlignItems,
    #[rebuild(layout)]
    pub align_content: Justify,
    #[rebuild(layout)]
    pub column_gap: f32,
    #[rebuild(layout)]
    pub row_gap: f32,
}

impl<T, V> Stack<T, V> {
    pub fn new(axis: Axis, content: V) -> Self {
        Self {
            content: PodSequence::new(content),
            size: None,
            axis,
            justify_content: Justify::Start,
            align_items: AlignItems::Start,
            align_content: Justify::Start,
            column_gap: 8.0,
            row_gap: 8.0,
        }
    }

    pub fn hstack(content: V) -> Self {
        Self::new(Axis::Horizontal, content)
    }

    pub fn vstack(content: V) -> Self {
        Self::new(Axis::Vertical, content)
    }

    pub fn justify_content(mut self, justify: impl Into<Justify>) -> Self {
        self.justify_content = justify.into();
        self
    }

    pub fn align_items(mut self, align: impl Into<AlignItems>) -> Self {
        self.align_items = align.into();
        self
    }

    pub fn center_items(mut self) -> Self {
        self.align_items = AlignItems::Center;
        self
    }

    pub fn stretch_items(mut self) -> Self {
        self.align_items = AlignItems::Stretch;
        self
    }

    pub fn align_content(mut self, align: impl Into<Justify>) -> Self {
        self.align_content = align.into();
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.column_gap = gap;
        self.row_gap = gap;
        self
    }

    pub fn column_gap(mut self, gap: f32) -> Self {
        self.column_gap = gap;
        self
    }

    pub fn row_gap(mut self, gap: f32) -> Self {
        self.row_gap = gap;
        self
    }
}

impl<T, V: ViewSequence<T>> Stack<T, V> {
    #[allow(clippy::too_many_arguments)]
    fn measure_fixed(
        &mut self,
        state: &mut StackState,
        content: &mut PodSequenceState<T, V>,
        data: &mut T,
        cx: &mut LayoutCx,
        gap_major: f32,
        max_major: f32,
        space: Space,
    ) {
        state.lines.clear();

        let mut major = 0.0;
        let mut minor = 0.0;
        let mut flex_sum = 0.0;

        let mut start = 0;

        for i in 0..self.content.len() {
            let size = self.content.layout(i, content, cx, data, space);
            let (child_major, child_minor) = self.axis.unpack(size);
            state.majors[i] = child_major;
            state.minors[i] = child_minor;

            let gap = if i > 0 { gap_major } else { 0.0 };

            if major + child_major + gap > max_major {
                state.lines.push(StackLine {
                    start,
                    end: i,
                    major,
                    minor,
                    flex_sum,
                });

                start = i;
                major = child_major;
                minor = child_minor;
                flex_sum = content[i].flex;
            } else {
                major += child_major + gap;
                minor = minor.max(child_minor);
                flex_sum += content[i].flex;
            }
        }

        state.lines.push(StackLine {
            start,
            end: self.content.len(),
            major,
            minor,
            flex_sum,
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn measure_flex(
        &mut self,
        state: &mut StackState,
        content: &mut PodSequenceState<T, V>,
        data: &mut T,
        cx: &mut LayoutCx,
        min_major: f32,
        max_major: f32,
        max_minor: f32,
    ) {
        for line in state.lines.iter_mut() {
            let overflow = line.major - max_major;
            let underflow = min_major - line.major;

            let px_per_flex = if overflow > 0.0 {
                -overflow / line.flex_sum
            } else if underflow > 0.0 {
                underflow / line.flex_sum
            } else {
                continue;
            };

            for i in line.start..line.end {
                let flex = content[i].flex;

                let is_stretch = self.align_items.is_stretch();

                if !is_stretch && flex == 0.0 {
                    continue;
                }

                let desired_major = state.majors[i] + px_per_flex * flex;

                let space = if is_stretch {
                    Space::new(
                        self.axis.pack(desired_major, line.minor),
                        self.axis.pack(desired_major, line.minor),
                    )
                } else {
                    Space::new(
                        self.axis.pack(desired_major, 0.0),
                        self.axis.pack(desired_major, max_minor),
                    )
                };

                let size = self.content.layout(i, content, cx, data, space);
                let (child_major, child_minor) = self.axis.unpack(size);

                line.major += child_major - state.majors[i];
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
    flex_sum: f32,
}

#[doc(hidden)]
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

impl<T, V: ViewSequence<T>> View<T> for Stack<T, V> {
    type State = (StackState, PodSequenceState<T, V>);

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

        for i in 0..self.content.len() {
            self.content.rebuild(i, content, cx, data, &old.content)
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
            self.content.event(i, content, cx, data, event);
        }
    }

    fn layout(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        let content_space = space;

        let (max_major, max_minor) = self.axis.unpack(content_space.max);
        let (min_major, min_minor) = self.axis.unpack(content_space.min);

        let (gap_major, gap_minor) = self.axis.unpack((self.column_gap, self.row_gap));

        self.measure_fixed(
            state,
            content,
            data,
            cx,
            gap_major,
            max_major,
            content_space,
        );
        self.measure_flex(state, content, data, cx, min_major, max_major, max_minor);

        let content_major = state.major().min(max_major);
        let content_minor = state.minor(gap_minor).max(min_minor);

        let content_size = self.axis.pack(content_major, content_minor);
        let size = content_size;

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
        scene: &mut Canvas,
    ) {
        for i in 0..self.content.len() {
            self.content.draw(i, content, cx, data, scene);
        }
    }
}
