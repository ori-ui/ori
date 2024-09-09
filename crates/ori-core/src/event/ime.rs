use std::ops::Range;

/// Input Method Editor (IME) state.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Ime {
    /// The current text being edited.
    pub text: String,

    /// The current selection range.
    pub selection: Range<usize>,

    /// The current composition range.
    pub compose: Option<Range<usize>>,
}
