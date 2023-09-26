use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
    hash::BuildHasher,
    mem,
    sync::Arc,
};

use super::SCALE_FACTOR;

use super::Key;

thread_local! {
    static THEME: RefCell<Theme> = Default::default();
}

impl<T: Any> Key<T> {
    /// Set a value in the global [`Theme`].
    pub fn set(self, value: impl Into<T>) {
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

/// Set a value in the global theme.
pub fn set_style<T: Any>(key: Key<T>, value: impl Into<T>) {
    key.set(value);
}

/// Extend the global theme.
pub fn set_theme(theme: impl Into<Theme>) {
    Theme::global(|global| global.extend(theme));
}

/// Get a value from the current theme.
pub fn style<T: Clone + Default + Any>(key: Key<T>) -> T {
    key.get()
}

/// Get a snapshot of the global theme.
pub fn theme_snapshot() -> Theme {
    Theme::global_snapshot()
}

/// Run a function with a temporary global theme.
///
/// This restores the previous global theme after the function returns.
pub fn themed<T>(f: impl FnOnce() -> T) -> T {
    let snapshot = Theme::global_snapshot();
    let result = f();
    Theme::make_global(snapshot);
    result
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
    Value(Arc<dyn Any>),
    Getter(Arc<dyn Any>),
}

impl Debug for ThemeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeEntry::Value(value) => write!(f, "Value({:?})", value),
            ThemeEntry::Getter(_) => write!(f, "Getter(...)"),
        }
    }
}

#[derive(Clone, Default)]
struct ThemeHasher;

impl BuildHasher for ThemeHasher {
    type Hasher = seahash::SeaHasher;

    fn build_hasher(&self) -> Self::Hasher {
        seahash::SeaHasher::new()
    }
}

/// A map of style values.
#[derive(Clone, Debug)]
pub struct Theme {
    values: HashMap<&'static str, ThemeEntry, ThemeHasher>,
}

impl Default for Theme {
    fn default() -> Self {
        Self::empty().with(SCALE_FACTOR, 1.0)
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
    pub fn set<T: Any>(&mut self, key: Key<T>, value: impl Into<T>) {
        let value = Arc::new(value.into());
        self.values.insert(key.name(), ThemeEntry::Value(value));
    }

    /// Map a value in the theme.
    pub fn map<T: Any>(&mut self, key: Key<T>, map: impl Fn(&Theme) -> T + 'static) {
        let map: Box<dyn Fn(&Theme) -> T> = Box::new(move |theme: &Theme| map(theme));
        (self.values).insert(key.name(), ThemeEntry::Getter(Arc::new(map)));
    }

    /// Set a value in the theme and return the theme.
    pub fn with<T: Any>(mut self, key: Key<T>, value: impl Into<T>) -> Self {
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

    fn try_get_inner<T: Clone + Any>(&self, name: &'static str) -> Result<T, ThemeError> {
        let entry = self.values.get(name).ok_or(ThemeError::MissingKey(name))?;
        match entry {
            ThemeEntry::Value(value) => Self::downcast(value.as_ref(), name).cloned(),
            ThemeEntry::Getter(getter) => {
                let getter = Self::downcast::<Box<dyn Fn(&Theme) -> T>>(getter.as_ref(), name)?;
                Ok(getter(self))
            }
        }
    }

    /// Get a value from the theme.
    pub fn try_get<T: Clone + Any>(&self, key: Key<T>) -> Option<T> {
        self.try_get_inner(key.name()).ok()
    }

    /// Get a value from the theme.
    pub fn get<T: Clone + Default + Any>(&self, key: Key<T>) -> T {
        match self.try_get_inner(key.name()) {
            Ok(value) => value,
            Err(err) => {
                crate::log::warn_internal!("{}", err);
                T::default()
            }
        }
    }

    /// Get a mutable reference to the global theme.
    pub fn global<T>(f: impl FnOnce(&mut Self) -> T) -> T {
        THEME.with(|theme| {
            let mut theme = theme.borrow_mut();
            f(&mut theme)
        })
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
