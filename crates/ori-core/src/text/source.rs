use std::path::{Path, PathBuf};

pub use ori_macro::include_font;

/// A source for a font.
#[derive(Clone, Debug)]
pub enum FontSource {
    /// A font loaded from data.
    Data(Vec<u8>),
    /// A font loaded from a file.
    Path(PathBuf),
    /// A collection of fonts.
    Set(Vec<FontSource>),
}

impl From<Vec<u8>> for FontSource {
    fn from(data: Vec<u8>) -> Self {
        Self::Data(data)
    }
}

impl From<&[u8]> for FontSource {
    fn from(data: &[u8]) -> Self {
        Self::Data(data.to_vec())
    }
}

impl From<&str> for FontSource {
    fn from(data: &str) -> Self {
        Self::Data(data.as_bytes().to_vec())
    }
}

impl From<&Path> for FontSource {
    fn from(path: &Path) -> Self {
        Self::Path(path.to_path_buf())
    }
}

impl From<PathBuf> for FontSource {
    fn from(path: PathBuf) -> Self {
        Self::Path(path)
    }
}

impl From<Vec<FontSource>> for FontSource {
    fn from(sources: Vec<FontSource>) -> Self {
        Self::Set(sources)
    }
}
