/// Event emitted to a view when its hot state changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HotChanged(pub bool);
