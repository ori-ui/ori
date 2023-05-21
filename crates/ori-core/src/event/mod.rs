mod cursor;
mod debug;
mod keyboard;
mod pointer;
mod window;

pub use cursor::*;
pub use debug::*;
pub use keyboard::*;
pub use pointer::*;
pub use window::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}
