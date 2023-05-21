use std::{iter, slice};

use deref_derive::{Deref, DerefMut};
use glam::Vec2;
use ori_graphics::Rect;
use ori_reactive::Event;
use smallvec::{smallvec, SmallVec};

use crate::{
    AlignItem, AnyView, AvailableSpace, Axis, Context, DrawContext, EventContext, IntoChildren,
    IntoNode, JustifyContent, LayoutContext, Node, Parent, View,
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
    pub align_items: AlignItem,
    /// The gap between the children.
    pub gap: f32,
}

impl Default for FlexLayout {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            axis: Axis::Vertical,
            justify_content: JustifyContent::Start,
            align_items: AlignItem::Start,
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

    pub fn from_style(cx: &mut LayoutContext) -> Self {
        let axis = cx.style::<Axis>("direction");
        let justify_content = cx.style("justify-content");
        let align_items = cx.style("align-items");
        let gap = cx.style_range("gap", 0.0..axis.major(cx.parent_space.max));

        Self {
            axis,
            justify_content,
            align_items,
            gap,
            ..Self::default()
        }
    }
}

/// Children of a [`View`].
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
        self.nodes.iter().map(SmallVec::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
        space: AvailableSpace,
        mut flex: FlexLayout,
    ) -> Vec2 {
        let padding = cx.state.padding;
        let padded_space = space.shrink(padding.size());
        flex.offset += padding.top_left();

        cx.with_space(padded_space, |cx| {
            self.flex_layout_padded(cx, padded_space, flex) + padding.size()
        })
    }

    /// Layout the children using a FlexLayout.
    fn flex_layout_padded(
        &self,
        cx: &mut LayoutContext,
        space: AvailableSpace,
        flex: FlexLayout,
    ) -> Vec2 {
        let FlexLayout {
            offset,
            axis,
            justify_content,
            align_items,
            gap,
        } = flex;

        // calculate the bounds of the major and minor axis
        let (min_major, min_minor) = axis.unpack(space.min);
        let (max_major, max_minor) = axis.unpack(space.max);

        let loosend_space = space.loosen();

        // initialize the major and minor axis
        let mut minor = min_minor;
        let mut major = self.len().saturating_sub(1) as f32 * gap;
        let mut flex_grow_sum = 0.0;
        let mut flex_shrink_sum = 0.0;

        // first we need to measure the fixed-sized children to determine their size
        //
        // NOTE: using a SmallVec here is a bit faster than using a Vec, but it's not a huge
        // difference
        let mut any_changed = false;
        let mut child_majors: SmallVec<[f32; 4]> = smallvec![0.0; self.len()];
        let mut child_flexes: SmallVec<[_; 4]> = smallvec![(None, None); self.len()];
        for (i, child) in self.iter().enumerate() {
            // get the flex grow and shrink factors
            let flex_grow = child.style::<Option<f32>>(cx, "flex-grow");
            let flex_shrink = child.style::<Option<f32>>(cx, "flex-shrink");

            // get the flex shorthand property
            let flex = child.style::<Option<f32>>(cx, "flex");
            let (flex_grow, flex_shrink) = match flex {
                Some(flex) => (
                    Some(flex_shrink.unwrap_or(flex)),
                    Some(flex_grow.unwrap_or(1.0)),
                ),
                None => (flex_grow, flex_shrink),
            };

            let is_flex = flex_grow.is_some() || flex_shrink.is_some();

            // add the flex grow and shrink factors to the sum
            flex_grow_sum += flex_grow.unwrap_or(0.0);
            flex_shrink_sum += flex_shrink.unwrap_or(0.0);

            // store the flex grow and shrink factors
            child_flexes[i] = (flex_grow, flex_shrink);

            // layout the child
            let needs_layout = child.needs_layout();
            let space_changed = child.space_changed(loosend_space);
            let size = if needs_layout || space_changed || any_changed {
                let old_size = child.size();

                let size = child.layout(cx, loosend_space);

                any_changed |= size != old_size;
                size
            } else {
                child.size()
            };

            let (child_major, child_minor) = axis.unpack(size);

            // store the size
            child_majors[i] = child_major;

            // update the major and minor axis
            major += child_major;

            if !is_flex {
                minor = minor.max(child_minor);
            }
        }

        // now we need to measure the flex-sized children to determine their size
        let remaining_major = max_major - major;
        let should_grow = remaining_major > 0.0;

        // calculate the amount of pixels per flex
        let px_per_flex = if should_grow {
            remaining_major / flex_grow_sum
        } else {
            remaining_major / flex_shrink_sum
        };

        for (i, child) in self.iter().enumerate() {
            // if the child has a flex property, now is the time
            let (flex_grow, flex_shrink) = child_flexes[i];
            if flex_grow.is_none() && should_grow || flex_shrink.is_none() && !should_grow {
                continue;
            }

            // calculate the desired size of the child
            let desired_major = if should_grow {
                child_majors[i] + px_per_flex * flex_grow.unwrap()
            } else {
                child_majors[i] + px_per_flex * flex_shrink.unwrap()
            };

            if desired_major == child_majors[i] {
                continue;
            }

            let child_space = AvailableSpace {
                min: axis.pack(desired_major, 0.0),
                max: axis.pack(desired_major, max_minor),
            };

            let size = child.relayout(cx, child_space);
            let (child_major, child_minor) = axis.unpack(size);

            // update the major and minor axis
            minor = minor.max(child_minor);
            major += child_major - child_majors[i];

            // store the size
            child_majors[i] = child_major;
        }

        // we need to re-measure the children to determine their size
        for (i, child) in self.iter().enumerate() {
            let align_self = child.style::<Option<AlignItem>>(cx, "align-self");

            if align_items != AlignItem::Stretch && align_self != Some(AlignItem::Stretch) {
                continue;
            }

            // calculate the constraints for the child
            let child_major = child_majors[i];
            let child_space = AvailableSpace {
                min: axis.pack(child_major, minor),
                max: axis.pack(child_major, minor),
            };

            // FIXME: calling layout again is not ideal, but it's the only way to get the
            // correct size for the child, since we don't know the minor size until we've
            // measured all the children
            let size = if any_changed {
                child.relayout(cx, child_space)
            } else {
                child.size()
            };

            child_majors[i] = axis.major(size);
        }

        major = major.max(min_major);

        let child_offsets = justify_content.justify(&child_majors, major, gap);

        // now we can layout the children
        for (child, align_major) in self.iter().zip(child_offsets) {
            // get the align item for the child
            let align_item = match child.style::<Option<AlignItem>>(cx, "align-self") {
                Some(align) => align,
                None => align_items,
            };

            // align the minor axis
            let child_minor = axis.minor(child.size());
            let align_minor = align_item.align(0.0, minor, child_minor);

            // set the offset for the child
            let child_offset = axis.pack(align_major, align_minor);
            child.set_offset(offset + child_offset);
        }

        // return the size of the flex container
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
    type IntoIter = iter::Flatten<smallvec::IntoIter<[SmallVec<[Self::Item; 1]>; 1]>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter().flatten()
    }
}

impl<'a, T: View> IntoIterator for &'a Children<T> {
    type Item = &'a Node<T>;
    type IntoIter = iter::Flatten<slice::Iter<'a, SmallVec<[Node<T>; 1]>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter().flatten()
    }
}

impl<'a> IntoIterator for &'a mut Children {
    type Item = &'a mut Node;
    type IntoIter = iter::Flatten<slice::IterMut<'a, SmallVec<[Node; 1]>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter_mut().flatten()
    }
}
