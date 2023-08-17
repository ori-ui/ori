/// The alignment of items along the cross axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AlignItems {
    /// Items are packed toward the start of the stack.
    Start,
    /// Items are packed toward the end of the stack.
    End,
    /// Items are packed toward the center of the stack.
    Center,
    /// Items are stretched to fill the stack.
    Stretch,
}

impl AlignItems {
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
    pub fn layout(
        self,
        sizes: impl ExactSizeIterator<Item = f32> + Clone,
        mut set_position: impl FnMut(usize, f32),
        size: f32,
        gap: f32,
    ) {
        if sizes.len() == 0 {
            return;
        }

        let total_gap = gap * (sizes.len() - 1) as f32;
        let total_size = sizes.clone().sum::<f32>() + total_gap;

        match self {
            Justify::Start => {
                let mut position = 0.0;

                for (i, size) in sizes.enumerate() {
                    set_position(i, position);
                    position += size + gap;
                }
            }
            Justify::Center => {
                let mut position = (size - total_size) / 2.0;

                for (i, size) in sizes.enumerate() {
                    set_position(i, position);
                    position += size + gap;
                }
            }
            Justify::End => {
                let mut position = size - total_size;

                for (i, size) in sizes.enumerate() {
                    set_position(i, position);
                    position += size + gap;
                }
            }
            Justify::SpaceBetween => {
                let gap = (size - total_size) / (sizes.len() - 1) as f32;
                let mut position = 0.0;

                for (i, size) in sizes.enumerate() {
                    set_position(i, position);
                    position += size + gap;
                }
            }
            Justify::SpaceAround => {
                let gap = (size - total_size) / sizes.len() as f32;
                let mut position = gap / 2.0;

                for (i, size) in sizes.enumerate() {
                    set_position(i, position);
                    position += size + gap;
                }
            }
            Justify::SpaceEvenly => {
                let gap = (size - total_size) / (sizes.len() + 1) as f32;
                let mut position = gap;

                for (i, size) in sizes.enumerate() {
                    set_position(i, position);
                    position += size + gap;
                }
            }
        }
    }
}
