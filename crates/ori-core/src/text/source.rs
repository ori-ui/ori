use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

pub use ori_macro::include_font;

/// A source for a font.
#[derive(Clone, Debug)]
pub enum FontSource<'a> {
    /// A font loaded from data.
    Data(Cow<'a, [u8]>),

    /// A font loaded from a file.
    Path(Cow<'a, Path>),

    /// A collection of fonts.
    Bundle(Cow<'a, [u8]>),
}

impl From<Vec<u8>> for FontSource<'_> {
    fn from(data: Vec<u8>) -> Self {
        Self::Data(Cow::Owned(data))
    }
}

impl<'a> From<&'a [u8]> for FontSource<'a> {
    fn from(data: &'a [u8]) -> Self {
        Self::Data(Cow::Borrowed(data))
    }
}

impl<'a> From<&'a str> for FontSource<'a> {
    fn from(data: &'a str) -> Self {
        Self::Path(Cow::Borrowed(Path::new(data)))
    }
}

impl<'a> From<&'a Path> for FontSource<'a> {
    fn from(path: &'a Path) -> Self {
        Self::Path(Cow::Borrowed(path))
    }
}

impl From<PathBuf> for FontSource<'_> {
    fn from(path: PathBuf) -> Self {
        Self::Path(Cow::Owned(path))
    }
}
