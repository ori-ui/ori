use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    fmt::Debug,
    hash::BuildHasher,
    mem,
    sync::Arc,
};

use super::Key;

/// Get a value from the current theme.
pub fn style<T: Clone + Default + Any>(key: impl AsRef<Key<T>>) -> T {
    Theme::context(|theme| theme.get(key))
}

/// Get a value in a [`Theme`].
#[derive(Clone, Debug)]
pub struct Style {
    type_id: TypeId,
    value: Arc<dyn Any>,
}

impl Style {
    /// Create a new style value.
    pub fn new<T: Any>(value: T) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            value: Arc::new(value),
        }
    }

    /// Downcast the value to a reference.
    #[inline(always)]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        if self.type_id == TypeId::of::<T>() {
            let ptr = self.value.as_ref() as *const dyn Any as *const T;

            // SAFETY: We just checked that the type ID is correct.
            unsafe { Some(&*ptr) }
        } else {
            None
        }
    }
}

/// A map of style values.
#[derive(Clone, Debug, Default)]
pub struct Theme {
    values: Arc<HashMap<&'static str, Style, ThemeHasher>>,
}

impl Theme {
    thread_local! {
        static CONTEXT: RefCell<Theme> = RefCell::new(Theme::default());
    }

    /// Create a new theme.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a value in the current theme.
    pub fn set<T: Any>(&mut self, key: impl AsRef<Key<T>>, value: T) {
        let key = key.as_ref().name();
        let value = Style::new(value);

        Arc::make_mut(&mut self.values).insert(key, value);
    }

    /// Get a value from the current theme.
    pub fn get<T: Clone + Default + Any>(&self, key: impl AsRef<Key<T>>) -> T {
        self.try_get(key).cloned().unwrap_or_default()
    }

    /// Try getting a value from the current theme.
    pub fn try_get<T: Any>(&self, key: impl AsRef<Key<T>>) -> Option<&T> {
        let key = key.as_ref().name();
        let value = self.values.get(key)?;

        value.downcast_ref()
    }

    /// Extend the theme with another theme.
    pub fn extend(&mut self, other: Self) {
        Arc::make_mut(&mut self.values).extend(other.values.as_ref().clone());
    }

    /// Run a function with the given theme as the current theme.
    pub fn as_context<T>(&mut self, f: impl FnOnce() -> T) -> T {
        Self::CONTEXT.with_borrow_mut(|context| mem::swap(context, self));
        let result = f();
        Self::CONTEXT.with_borrow_mut(|context| mem::swap(context, self));
        result
    }

    /// Get the current theme.
    pub fn context<T>(f: impl FnOnce(&mut Self) -> T) -> T {
        Self::CONTEXT.with_borrow_mut(f)
    }

    /// Get a snapshot of the current theme.
    pub fn snapshot() -> Self {
        Self::context(|theme| theme.clone())
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
