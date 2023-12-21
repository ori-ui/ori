use std::{
    any::Any,
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
    hash::BuildHasher,
    mem,
    sync::Arc,
};

use crate::log::warn_internal;

use super::SCALE_FACTOR;

use super::Key;

impl<T: Any> Key<T> {
    /// Set a value in the global [`Theme`].
    pub fn set(self, value: impl Into<T>) {
        Theme::global(|theme| theme.set(self, value));
    }

    /// Get a value from the global [`Theme`].
    pub fn get(self) -> T
    where
        T: Clone + Default,
    {
        Theme::GLOBAL.with(|theme| theme.borrow().get(self))
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
pub fn style<T: Clone + Default + Any>(key: impl AsRef<Key<T>>) -> T {
    key.as_ref().get()
}

/// Get a snapshot of the global theme.
pub fn theme_snapshot() -> Theme {
    Theme::snapshot()
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
    values: Arc<HashMap<Cow<'static, str>, ThemeEntry, ThemeHasher>>,
}

impl Default for Theme {
    fn default() -> Self {
        Self::empty().with(SCALE_FACTOR, 1.0)
    }
}

impl Theme {
    thread_local! {
        /// The global theme.
        ///
        /// This is used by [`style`](crate::style) and [`set_style`](crate::set_style).
        pub static GLOBAL: RefCell<Theme> = Default::default();
    }

    fn empty() -> Self {
        Self {
            values: Default::default(),
        }
    }

    fn values_mut(&mut self) -> &mut HashMap<Cow<'static, str>, ThemeEntry, ThemeHasher> {
        Arc::make_mut(&mut self.values)
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
        (self.values_mut()).insert(Cow::Borrowed(key.name()), ThemeEntry::Value(value));
    }

    /// Map a value in the theme.
    pub fn map<T: Any>(&mut self, key: Key<T>, map: impl Fn(&Theme) -> T + 'static) {
        let map: Box<dyn Fn(&Theme) -> T> = Box::new(move |theme: &Theme| map(theme));
        (self.values_mut()).insert(Cow::Borrowed(key.name()), ThemeEntry::Getter(Arc::new(map)));
    }

    /// Set a value in the theme and return the theme.
    pub fn with<T: Any>(mut self, key: Key<T>, value: impl Into<T>) -> Self {
        self.set(key, value);
        self
    }

    /// Extend the theme with another theme.
    pub fn extend(&mut self, other: impl Into<Self>) {
        let other = match Arc::try_unwrap(other.into().values) {
            Ok(other) => other,
            Err(other) => other.as_ref().clone(),
        };

        self.values_mut().extend(other);
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
    pub fn try_get<T: Clone + Any>(&self, key: impl AsRef<Key<T>>) -> Option<T> {
        self.try_get_inner(key.as_ref().name()).ok()
    }

    /// Get a value from the theme.
    pub fn get<T: Clone + Default + Any>(&self, key: impl AsRef<Key<T>>) -> T {
        match self.try_get_inner(key.as_ref().name()) {
            Ok(value) => value,
            Err(err) => {
                warn_internal!("{}", err);
                T::default()
            }
        }
    }

    /// Get a mutable reference to the global theme.
    pub fn global<T>(f: impl FnOnce(&mut Self) -> T) -> T {
        Self::GLOBAL.with(|theme| f(&mut *theme.borrow_mut()))
    }

    /// Get a snapshot of the global theme.
    pub fn snapshot() -> Self {
        Self::GLOBAL.with(|theme| theme.borrow().clone())
    }

    /// Swap this theme with the global theme.
    pub fn swap_global(this: &mut Self) {
        Self::global(|theme| mem::swap(&mut *theme, this));
    }

    /// Set this theme as the global theme.
    pub fn set_global(this: Self) -> Self {
        Self::global(|theme| mem::replace(&mut *theme, this))
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
