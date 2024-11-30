use std::{
    borrow::Cow,
    fs,
    io::{self, Cursor, Read},
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

    /// A zlib-compressed bundle of fonts.
    Bundle(Cow<'a, [u8]>),
}

impl<'a> FontSource<'a> {
    /// Get the data of the font source.
    pub fn data(&self) -> io::Result<Vec<Cow<'a, [u8]>>> {
        match self {
            Self::Data(data) => Ok(vec![data.clone()]),
            Self::Path(path) => Ok(vec![Cow::Owned(fs::read(path.as_ref())?)]),
            Self::Bundle(data) => {
                let data = miniz_oxide::inflate::decompress_to_vec(data).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "failed to decompress font bundle",
                    )
                })?;

                let mut cursor = Cursor::new(&data);

                let mut count = [0; 4];
                cursor.read_exact(&mut count)?;

                let count = u32::from_le_bytes(count) as usize;

                let mut fonts = Vec::with_capacity(count);

                for _ in 0..count {
                    let mut size = [0; 4];
                    cursor.read_exact(&mut size)?;

                    let size = u32::from_le_bytes(size) as usize;

                    let mut data = vec![0; size];
                    cursor.read_exact(&mut data)?;

                    fonts.push(Cow::Owned(data));
                }

                assert_eq!(cursor.position() as usize, data.len());

                Ok(fonts)
            }
        }
    }
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
