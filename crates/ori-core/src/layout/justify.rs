use core::slice;

/// The alignment of items along the cross axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Align {
    /// Items are packed toward the start of the stack.
    Start,
    /// Items are packed toward the end of the stack.
    End,
    /// Items are packed toward the center of the stack.
    Center,
    /// Items are stretched to fill the stack.
    Stretch,
}

impl Align {
    /// Returns true if the alignment is stretch.
    pub const fn is_stretch(&self) -> bool {
        matches!(self, Self::Stretch)
    }

    /// Aligns an item within the given space.
    pub fn align(self, available: f32, size: f32) -> f32 {
        match self {
            Self::Start => 0.0,
            Self::End => available - size,
            Self::Center => (available - size) / 2.0,
            Self::Stretch => 0.0,
        }
    }
}

/// The justify content of a stack container.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Justify {
    /// Items are packed toward the start of the stack.
    Start,
    /// Items are packed toward the end of the stack.
    End,
    /// Items are packed toward the center of the stack.
    Center,
    /// Items are evenly distributed in the stack, with equal-size spaces between them.
    SpaceBetween,
    /// Items are evenly distributed in the stack, with half-size spaces on either end.
    SpaceAround,
    /// Items are evenly distributed in the stack.
    SpaceEvenly,
}

impl Justify {
    /// Layout the items in a stack container.
    pub fn layout(self, sizes: &[f32], size: f32, gap: f32) -> JustifyIterator {
        let count = sizes.len() as f32;

        let total_gap = gap * (count - 1.0);
        let total_size = sizes.iter().sum::<f32>() + total_gap;

        let gap = match self {
            Self::Start | Self::End | Self::Center => gap,
            Self::SpaceBetween => (size - (total_size - total_gap)) / (count - 1.0),
            Self::SpaceAround => (size - (total_size - total_gap)) / count,
            Self::SpaceEvenly => (size - (total_size - total_gap)) / (count + 1.0),
        };

        let position = match self {
            Self::Start | Self::SpaceBetween => 0.0,
            Self::Center => (size - total_size) / 2.0,
            Self::End => size - total_size,
            Self::SpaceAround => gap / 2.0,
            Self::SpaceEvenly => gap,
        };

        JustifyIterator {
            sizes: sizes.iter(),
            position,
            gap,
        }
    }
}

/// An iterator over the positions of items in a stack container.
pub struct JustifyIterator<'a> {
    sizes: slice::Iter<'a, f32>,
    position: f32,
    gap: f32,
}

impl<'a> Iterator for JustifyIterator<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let position = self.position;
        self.position += *self.sizes.next()? + self.gap;
        Some(position)
    }
}
