//! Provides styling for the `ori` crate.

mod attribute;
mod cache;
mod loader;
mod parse;
mod selector;
mod sheet;
mod style;
mod styled;
mod transition;

pub use attribute::*;
pub use cache::*;
pub use loader::*;
pub use parse::*;
pub use selector::*;
pub use sheet::*;
pub use style::*;
pub use styled::*;
pub use transition::*;

pub mod prelude {
    //! A collection of commonly used types and traits.

    pub use crate::attribute::{trans, Length, Length::*, StyleAttribute, StyleAttributeValue};
    pub use crate::sheet::Stylesheet;
    pub use crate::style::Style;
    pub use crate::styled::{Stylable, Styled};

    pub use crate::include_stylesheet;
}
