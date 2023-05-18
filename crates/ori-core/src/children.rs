use deref_derive::{Deref, DerefMut};
use glam::Vec2;
use ori_graphics::Rect;
use smallvec::SmallVec;

use crate::{
    AlignItems, AnyView, Axis, BoxConstraints, DrawContext, Event, EventContext, IntoChildren,
    IntoNode, JustifyContent, LayoutContext, Node, Padding, Parent, View,
};

/// A layout that lays out children in a flexbox-like manner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlexLayout {
    /// The offset to apply to the children.
    ///
    /// This is useful for scrolling or padding.
    pub offset: Vec2,
    /// The axis to use for laying out the children.
    pub axis: Axis,
    /// The justification of the children.
    pub justify_content: JustifyContent,
    /// The alignment of the children.
    pub align_items: AlignItems,
    /// The gap between the children.
    pub gap: f32,
}

impl Default for FlexLayout {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            axis: Axis::Vertical,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            gap: 0.0,
        }
    }
}

impl FlexLayout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn vertical() -> Self {
        Self {
            axis: Axis::Vertical,
            ..Self::default()
        }
    }

    pub fn horizontal() -> Self {
        Self {
            axis: Axis::Horizontal,
            ..Self::default()
        }
    }

    pub fn row() -> Self {
        Self::horizontal()
    }

    pub fn column() -> Self {
        Self::vertical()
    }
}

#[derive(Deref, DerefMut)]
pub struct Children<T: View = Box<dyn AnyView>> {
    nodes: SmallVec<[SmallVec<[Node<T>; 1]>; 1]>,
}

impl<T: View> Default for Children<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: View> Parent for Children<T> {
    type Child = T;

    fn clear_children(&mut self) {
        self.nodes.clear();
    }

    fn add_child<I: IntoIterator, U: ?Sized>(&mut self, child: impl IntoChildren<I>) -> usize
    where
        I::Item: IntoNode<Self::Child, U>,
    {
        let children = child
            .into_children()
            .into_iter()
            .map(IntoNode::into_node)
            .collect();

        let index = self.nodes.len();
        self.nodes.push(children);
        index
    }

    fn set_child<I: IntoIterator, U: ?Sized>(&mut self, index: usize, child: impl IntoChildren<I>)
    where
        I::Item: IntoNode<Self::Child, U>,
    {
        let children = child
            .into_children()
            .into_iter()
            .map(IntoNode::into_node)
            .collect();

        self.nodes[index] = children;
    }
}

impl<T: View> Children<T> {
    pub const fn new() -> Self {
        Self {
            nodes: SmallVec::new_const(),
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Call the `event` method on all the children.
    pub fn event(&self, cx: &mut EventContext, event: &Event) {
        for child in self.iter() {
            child.event(cx, event);
        }
    }

    /// Layout the children using a FlexLayout.
    pub fn flex_layout(
        &self,
        cx: &mut LayoutContext,
        bc: BoxConstraints,
        mut flex: FlexLayout,
    ) -> Vec2 {
        let padding = Padding::from_style(cx, bc);
        let padded_bc = bc.shrink(padding.size());
        flex.offset += padding.top_left();
        self.flex_layout_padded(cx, padded_bc, flex) + padding.size()
    }

    /// Layout the children using a FlexLayout.
    fn flex_layout_padded(
        &self,
        cx: &mut LayoutContext,
        bc: BoxConstraints,
        flex: FlexLayout,
    ) -> Vec2 {
        let FlexLayout {
            offset,
            axis,
            justify_content,
            align_items,
            gap,
        } = flex;

        let max_minor = axis.minor(bc.max);
        let min_minor = axis.minor(bc.min);

        let max_major = axis.major(bc.max);
        let min_major = axis.major(bc.min);

        let mut minor = min_minor;
        let mut major = 0.0f32;

        // first we need to measure the children to determine their size
        //
        // NOTE: using a SmallVec here is a bit faster than using a Vec, but it's not a huge
        // difference
        let mut children = SmallVec::<[f32; 4]>::with_capacity(self.len());
        for (i, child) in self.iter().enumerate() {
            let child_bc = BoxConstraints {
                min: axis.pack(0.0, 0.0),
                max: axis.pack(max_major - major, max_minor),
            };
            let needs_layout = child.needs_layout();
            let size = child.layout(cx, child_bc);
            let child_minor = axis.minor(size);
            let child_major = axis.major(size);

            children.push(child_major);

            minor = minor.max(child_minor);
            major += child_major;

            if align_items == AlignItems::Stretch && needs_layout {
                child.request_layout();
            }

            if i > 0 {
                major += gap;
            }
        }

        if align_items == AlignItems::Stretch {
            // we need to re-measure the children to determine their size
            major = 0.0;
            children.clear();

            for (i, child) in self.iter().enumerate() {
                let child_bc = BoxConstraints {
                    min: axis.pack(0.0, minor),
                    max: axis.pack(max_major, minor),
                };
                // FIXME: calling layout again is not ideal, but it's the only way to get the
                // correct size for the child, since we don't know the minor size until we've
                // measured all the children
                let size = child.layout(cx, child_bc);
                let child_major = axis.major(size);

                children.push(child_major);

                major += child_major;

                if i > 0 {
                    major += gap;
                }
            }
        }

        major = major.max(min_major);

        let child_offsets = justify_content.justify(&children, major, gap);

        // now we can layout the children
        for (child, align_major) in self.iter().zip(child_offsets) {
            let child_minor = axis.minor(child.size());
            let align_minor = align_items.align(0.0, minor, child_minor);

            let child_offset = axis.pack(align_major, align_minor);
            child.set_offset(offset + child_offset);
        }

        axis.pack(major, minor)
    }

    pub fn local_rect(&self) -> Rect {
        let mut rect = None;

        for child in self.iter() {
            let rect = rect.get_or_insert_with(|| child.local_rect());
            *rect = rect.union(child.local_rect());
        }

        rect.unwrap_or_default()
    }

    pub fn rect(&self) -> Rect {
        let mut rect = None;

        for child in self.iter() {
            let rect = rect.get_or_insert_with(|| child.global_rect());
            *rect = rect.union(child.global_rect());
        }

        rect.unwrap_or_default()
    }

    pub fn size(&self) -> Vec2 {
        self.rect().size()
    }

    pub fn set_offset(&self, offset: Vec2) {
        if self.is_empty() {
            return;
        }

        let min = self.local_rect().min;

        for child in self.iter() {
            let child_offset = child.local_rect().min - min;
            child.set_offset(offset + child_offset);
        }
    }

    pub fn draw(&self, cx: &mut DrawContext) {
        for child in self.iter() {
            child.draw(cx);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Node<T>> {
        self.nodes.iter().flatten()
    }
}

impl<T: View> IntoIterator for Children<T> {
    type Item = Node<T>;
    type IntoIter = std::iter::Flatten<smallvec::IntoIter<[SmallVec<[Self::Item; 1]>; 1]>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter().flatten()
    }
}

impl<'a, T: View> IntoIterator for &'a Children<T> {
    type Item = &'a Node<T>;
    type IntoIter = std::iter::Flatten<std::slice::Iter<'a, SmallVec<[Node<T>; 1]>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter().flatten()
    }
}

impl<'a> IntoIterator for &'a mut Children {
    type Item = &'a mut Node;
    type IntoIter = std::iter::Flatten<std::slice::IterMut<'a, SmallVec<[Node; 1]>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter_mut().flatten()
    }
}
