use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
    hash::BuildHasher,
    mem,
    sync::Arc,
};

use crate::TEXT_SIZE;

use super::Key;

thread_local! {
    static THEME: RefCell<Theme> = Default::default();
}

impl<T: Any> Key<T> {
    /// Set a value in the global [`Theme`].
    pub fn set(self, value: impl Into<Style<T>>) {
        THEME.with(|theme| theme.borrow_mut().set(self, value));
    }

    /// Get a value from the global [`Theme`].
    pub fn get(self) -> T
    where
        T: Clone + Default,
    {
        THEME.with(|theme| theme.borrow().get(self))
    }
}

/// Set a value in the current theme.
pub fn set_style<T: Any>(key: Key<T>, value: impl Into<Style<T>>) {
    key.set(value);
}

/// Get a value from the current theme.
pub fn style<T: Clone + Default + Any>(key: Key<T>) -> T {
    key.get()
}

/// Run a function with a temporary global theme.
///
/// This restores the previous global theme after the function returns.
pub fn styled<T>(f: impl FnOnce() -> T) -> T {
    let snapshot = Theme::global_snapshot();
    let result = f();
    Theme::make_global(snapshot);
    result
}

#[derive(Default)]
struct ThemeHasher;

impl BuildHasher for ThemeHasher {
    type Hasher = seahash::SeaHasher;

    fn build_hasher(&self) -> Self::Hasher {
        seahash::SeaHasher::new()
    }
}

/// A value that in a [`Theme`].
#[derive(Clone, Debug)]
pub enum Style<T> {
    /// A value.
    Val(T),
    /// A key.
    Key(Key<T>),
}

impl<T> From<T> for Style<T> {
    fn from(value: T) -> Self {
        Self::Val(value)
    }
}

impl<T> From<Key<T>> for Style<T> {
    fn from(key: Key<T>) -> Self {
        Self::Key(key)
    }
}

#[derive(Clone, Debug)]
enum ThemeError {
    MissingKey(&'static str),
    WrongType(&'static str),
}

impl Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeError::MissingKey(key) => write!(f, "missing theme key `{}`", key),
            ThemeError::WrongType(key) => write!(f, "wrong theme type for `{}`", key),
        }
    }
}

#[derive(Clone)]
enum ThemeEntry {
    Val(Arc<dyn Any>),
    Key(&'static str),
}

impl Debug for ThemeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Val(_) => write!(f, "value"),
            Self::Key(key) => write!(f, "\"{}\"", key),
        }
    }
}

/// A map of style values.
#[derive(Clone, Debug)]
pub struct Theme {
    values: HashMap<&'static str, ThemeEntry>,
}

impl Default for Theme {
    fn default() -> Self {
        Self::empty().with(TEXT_SIZE, 16.0)
    }
}

impl Theme {
    fn empty() -> Self {
        Self {
            values: Default::default(),
        }
    }

    /// Create a new theme.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the number of values in the theme.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Get whether the theme is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Set a value in the theme.
    pub fn set<T: Any>(&mut self, key: Key<T>, value: impl Into<Style<T>>) {
        let entry = match value.into() {
            Style::Val(value) => ThemeEntry::Val(Arc::new(value)),
            Style::Key(key) => ThemeEntry::Key(key.name()),
        };

        self.values.insert(key.name(), entry);
    }

    /// Set a value in the theme and return the theme.
    pub fn with<T: Any>(mut self, key: Key<T>, value: impl Into<Style<T>>) -> Self {
        self.set(key, value);
        self
    }

    /// Extend the theme with another theme.
    pub fn extend(&mut self, other: impl Into<Self>) {
        self.values.extend(other.into().values);
    }

    fn downcast<'a, T: Any>(value: &'a dyn Any, name: &'static str) -> Result<&'a T, ThemeError> {
        value.downcast_ref().ok_or(ThemeError::WrongType(name))
    }

    fn try_get_inner<T: Any>(&self, mut name: &'static str) -> Result<&T, ThemeError> {
        loop {
            let entry = self.values.get(name).ok_or(ThemeError::MissingKey(name))?;
            match entry {
                ThemeEntry::Val(value) => {
                    return Self::downcast(value.as_ref(), name);
                }
                ThemeEntry::Key(key) => {
                    name = key;
                }
            }
        }
    }

    /// Get a value from the theme.
    pub fn try_get<T: Any>(&self, key: Key<T>) -> Option<&T> {
        self.try_get_inner(key.name()).ok()
    }

    /// Get a value from the theme.
    pub fn get<T: Clone + Default + Any>(&self, key: Key<T>) -> T {
        match self.try_get_inner(key.name()).cloned() {
            Ok(value) => value,
            Err(err) => {
                tracing::warn!("{}", err);
                println!("{}", err);
                T::default()
            }
        }
    }

    /// Get a mutable reference to the global theme.
    pub fn global(f: impl FnOnce(&mut Self)) {
        THEME.with(|theme| {
            let mut theme = theme.borrow_mut();
            f(&mut theme);
        });
    }

    /// Get a snapshot of the global theme.
    pub fn global_snapshot() -> Self {
        THEME.with(|theme| theme.borrow().clone())
    }

    /// Make this theme the global theme.
    ///
    /// This returns the previous global theme.
    pub fn make_global(mut this: Self) -> Self {
        THEME.with(|theme| {
            let mut theme = theme.borrow_mut();
            mem::swap(&mut *theme, &mut this);
        });

        this
    }

    /// Swap this theme with the global theme.
    pub fn swap_global(this: &mut Self) {
        THEME.with(|theme| {
            let mut theme = theme.borrow_mut();
            mem::swap(&mut *theme, this);
        });
    }

    /// Run a function with this theme as the global theme.
    ///
    /// This restores the previous global theme after the function returns.
    pub fn with_global<T>(this: &mut Self, f: impl FnOnce() -> T) -> T {
        Self::swap_global(this);
        let result = f();
        Self::swap_global(this);
        result
    }
}
