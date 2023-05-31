use std::{
    cmp::Ordering,
    ops::{Add, AddAssign},
};

/// The specificity of a [`StyleSelectors`](crate::StyleSelector).
///
/// See [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/Specificity) for more information.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StyleSpec {
    /// The number of classes in the selector.
    pub class: u16,
    /// The number of tags in the selector.
    pub tag: u16,
}

impl StyleSpec {
    #[allow(missing_docs)]
    pub const MAX: Self = Self::new(u16::MAX, u16::MAX);
    #[allow(missing_docs)]
    pub const INLINE: Self = Self::MAX;

    /// Create a new [`StyleSpec`].
    pub const fn new(class: u16, tag: u16) -> Self {
        Self { class, tag }
    }
}

impl PartialOrd for StyleSpec {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.class.partial_cmp(&other.class) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }
        self.tag.partial_cmp(&other.tag)
    }
}

impl Ord for StyleSpec {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.class.cmp(&other.class) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.tag.cmp(&other.tag)
    }
}

impl Add for StyleSpec {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            class: self.class + other.class,
            tag: self.tag + other.tag,
        }
    }
}

impl AddAssign for StyleSpec {
    fn add_assign(&mut self, other: Self) {
        self.class += other.class;
        self.tag += other.tag;
    }
}
