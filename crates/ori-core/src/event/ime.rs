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

    /// Whether the IME is multiline.
    pub multiline: bool,

    /// How the IME should capitalize text.
    pub capitalize: Capitalize,
}

/// Input Method Editor (IME) capitalization.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Capitalize {
    /// No capitalization.
    None,

    /// Capitalize the first letter of each word.
    Words,

    /// Capitalize the first letter of each sentence (default).
    Sentences,

    /// Capitalize all letters.
    All,
}

impl Default for Capitalize {
    fn default() -> Self {
        Self::Sentences
    }
}
