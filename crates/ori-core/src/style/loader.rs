use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::{StyleLoadError, Stylesheet};

/// A style that has been loaded from a file.
#[derive(Clone, Debug)]
pub struct LoadedStyle {
    modified: SystemTime,
    path: PathBuf,
    style: Stylesheet,
}

impl LoadedStyle {
    /// Loads a style from a file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, StyleLoadError> {
        let modified = path.as_ref().metadata()?.modified()?;
        let style = Stylesheet::load(&path)?;
        Ok(Self {
            modified,
            path: path.as_ref().to_path_buf(),
            style,
        })
    }

    /// Reloads the style if the file has been modified.
    ///
    /// Returns true if the style was reloaded.
    pub fn reload(&mut self) -> Result<bool, StyleLoadError> {
        let modified = self.path.metadata()?.modified()?;

        let needs_reload = modified > self.modified;
        if needs_reload {
            self.modified = modified;
            self.style = Stylesheet::load(&self.path)?;
        }

        Ok(needs_reload)
    }
}

/// A style that can be loaded from a file or be an inline style.
#[derive(Clone, Debug)]
pub enum LoadedStyleKind {
    /// A style that has been loaded from a file.
    Loaded(LoadedStyle),
    /// An inline style.
    Inline(Stylesheet),
}

impl LoadedStyleKind {
    /// Reloads the style if the file has been modified.
    ///
    /// Returns true if the style was reloaded.
    pub fn reload(&mut self) -> Result<bool, StyleLoadError> {
        match self {
            Self::Loaded(loaded) => loaded.reload(),
            Self::Inline(_) => Ok(false),
        }
    }

    /// Returns the style.
    pub fn style(&self) -> &Stylesheet {
        match self {
            Self::Loaded(style) => &style.style,
            Self::Inline(style) => style,
        }
    }
}

impl From<Stylesheet> for LoadedStyleKind {
    fn from(style: Stylesheet) -> Self {
        Self::Inline(style)
    }
}

impl TryFrom<&str> for LoadedStyleKind {
    type Error = StyleLoadError;

    fn try_from(path: &str) -> Result<Self, Self::Error> {
        Ok(Self::Loaded(LoadedStyle::load(path)?))
    }
}

impl TryFrom<&Path> for LoadedStyleKind {
    type Error = StyleLoadError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        Ok(Self::Loaded(LoadedStyle::load(path)?))
    }
}

/// A style loader that can load styles from files and inline styles.
///
/// Styles that are loaded from files are reloaded when the file is modified.
#[derive(Clone, Default, Debug)]
pub struct StyleLoader {
    styles: Vec<LoadedStyleKind>,
    cache: Stylesheet,
}

impl StyleLoader {
    /// Creates a new style loader.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears the loader.
    pub fn clear(&mut self) {
        self.styles.clear();
        self.cache = Stylesheet::new();
    }

    /// Adds a style to the loader.
    pub fn add_style<T: TryInto<LoadedStyleKind>>(&mut self, style: T) -> Result<(), T::Error> {
        self.styles.push(style.try_into()?);
        self.compute_cache();
        Ok(())
    }

    /// Recomputes the cache.
    fn compute_cache(&mut self) {
        self.cache = Stylesheet::new();

        for style in self.styles.iter() {
            self.cache.extend(style.style().clone());
        }
    }

    /// Reloads the styles if the files have been modified.
    pub fn reload(&mut self) -> Result<bool, StyleLoadError> {
        let mut needs_reload = false;

        for style in self.styles.iter_mut() {
            if style.reload()? {
                needs_reload = true;
            }
        }

        if needs_reload {
            self.compute_cache();
        }

        Ok(needs_reload)
    }

    /// Returns the style.
    pub fn stylesheet(&self) -> &Stylesheet {
        &self.cache
    }
}
