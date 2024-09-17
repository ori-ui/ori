use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    hash::{BuildHasherDefault, Hasher},
    marker::PhantomData,
    sync::Arc,
};

#[derive(Clone)]
enum StyleEntry {
    Value(TypeId, Arc<dyn Any>),
    Key(u64),
}

#[repr(transparent)]
#[derive(Clone)]
struct StylesHasher(u64);

impl Default for StylesHasher {
    #[inline(always)]
    fn default() -> Self {
        Self(0)
    }
}

impl Hasher for StylesHasher {
    fn write(&mut self, _bytes: &[u8]) {
        unreachable!()
    }

    #[inline(always)]
    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }

    #[inline(always)]
    fn finish(&self) -> u64 {
        self.0
    }
}

enum GetRefError {
    TypeMismatch,
    KeyNotFound,
}

/// A collection of styles.
#[derive(Clone, Default)]
pub struct Styles {
    styles: Arc<HashMap<u64, StyleEntry, BuildHasherDefault<StylesHasher>>>,
}

impl Styles {
    /// Create a new [`Styles`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the number of styles.
    pub fn len(&self) -> usize {
        self.styles.len()
    }

    /// Check if the styles are empty.
    pub fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }

    /// Insert a styled value.
    pub fn insert<T: 'static>(&mut self, key: Style<T>, styled: impl Into<Styled<T>>) {
        match styled.into() {
            Styled::Value(value) => self.insert_value(key, value),
            Styled::Style(style) => {
                let value = StyleEntry::Key(style.key);
                Arc::make_mut(&mut self.styles).insert(style.key, value);
            }
            Styled::Computed(derived) => self.insert_value(key, derived(self)),
        }
    }

    /// Insert a style.
    pub fn insert_value<T: 'static>(&mut self, key: Style<T>, style: T) {
        let entry = StyleEntry::Value(TypeId::of::<T>(), Arc::new(style));
        Arc::make_mut(&mut self.styles).insert(key.key, entry);
    }

    /// Insert a style key.
    pub fn insert_style<T: 'static>(&mut self, key: Style<T>, style: Style<T>) {
        self.insert_style_keys(key.key, style.key);
    }

    /// Insert a style key.
    pub fn insert_style_keys(&mut self, key: u64, style: u64) {
        Arc::make_mut(&mut self.styles).insert(key, StyleEntry::Key(style));
    }

    /// Insert a style key.
    pub fn with<T: 'static>(mut self, key: Style<T>, styled: impl Into<Styled<T>>) -> Self {
        self.insert(key, styled);
        self
    }

    /// Insert a styled value.
    pub fn with_value<T: 'static>(mut self, key: Style<T>, value: T) -> Self {
        self.insert_value(key, value);
        self
    }

    /// Insert a style key.
    pub fn with_style<T: 'static>(mut self, key: Style<T>, style: Style<T>) -> Self {
        self.insert_style(key, style);
        self
    }

    /// Extend the styles with another collection of styles.
    pub fn extend(&mut self, styles: impl Into<Styles>) {
        let styles = Arc::unwrap_or_clone(styles.into().styles);
        Arc::make_mut(&mut self.styles).extend(styles);
    }

    #[inline(always)]
    fn get_ref<T>(&self, key: u64) -> Result<&T, GetRefError>
    where
        T: 'static,
    {
        let style = self.styles.get(&key).ok_or(GetRefError::KeyNotFound)?;

        match style {
            StyleEntry::Value(type_id, value) => {
                if *type_id != TypeId::of::<T>() {
                    return Err(GetRefError::TypeMismatch);
                }

                debug_assert!(
                    value.is::<T>(),
                    "style is of type `{}",
                    std::any::type_name::<T>()
                );

                let ptr = value.as_ref() as *const _ as *const _;

                // SAFETY: The was just asserted to be of type `T`.
                unsafe { Ok(&*ptr) }
            }
            StyleEntry::Key(key) => self.get_ref(*key),
        }
    }

    /// Get a style.
    #[track_caller]
    #[inline(always)]
    pub fn get<T>(&self, style: Style<T>) -> Option<T>
    where
        T: Clone + 'static,
    {
        match self.get_ref::<T>(style.key) {
            Ok(value) => Some(value.clone()),
            Err(GetRefError::TypeMismatch) => {
                panic!(
                    "style is of a different type than `{}`",
                    std::any::type_name::<T>()
                )
            }
            Err(GetRefError::KeyNotFound) => None,
        }
    }

    /// Get a style, or a default value.
    #[track_caller]
    #[inline(always)]
    pub fn get_or<T>(&self, default: T, style: Style<T>) -> T
    where
        T: Clone + 'static,
    {
        self.get(style).unwrap_or(default)
    }

    /// Get a style, or a default value.
    #[track_caller]
    #[inline(always)]
    pub fn get_or_else<T, F>(&self, default: F, style: Style<T>) -> T
    where
        T: Clone + 'static,
        F: FnOnce() -> T,
    {
        self.get(style).unwrap_or_else(default)
    }
}

/// A style.
pub struct Style<T: ?Sized> {
    key: u64,
    marker: PhantomData<fn(&T)>,
}

impl<T: ?Sized> Style<T> {
    /// Create a new style.
    #[inline(always)]
    pub const fn new(key: &str) -> Self {
        Self {
            key: hash_style_key(key.as_bytes()),
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Clone for Style<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for Style<T> {}

/// Create a style value.
pub fn val<T>(val: impl Into<T>) -> Styled<T> {
    Styled::Value(val.into())
}

/// Create a style key.
pub const fn key<T>(key: &str) -> Styled<T> {
    Styled::Style(Style::new(key))
}

/// Create a computed style.
pub fn comp<T>(f: impl Fn(&Styles) -> T + Send + Sync + 'static) -> Styled<T> {
    Styled::Computed(Arc::new(Box::new(f)))
}

// Box<dyn Fn()> is 16 bytes large, however Arc<Box<dyn Fn()>> is only 8 bytes. since computed
// styles are used so infrequently, compared to the other variants, it's worth the tradeoff to save
// memory, even if it costs an extra indirection.
type Computed<T> = Arc<Box<dyn Fn(&Styles) -> T + Send + Sync>>;

/// A styled value.
#[derive(Clone)]
pub enum Styled<T> {
    /// A value.
    Value(T),

    /// A style key.
    Style(Style<T>),

    /// A derived style.
    Computed(Computed<T>),
}

impl<T> Styled<T> {
    /// Get the value, or a style from the styles.
    #[inline(always)]
    pub fn get(&self, styles: &Styles) -> Option<T>
    where
        T: Clone + 'static,
    {
        match self {
            Self::Value(value) => Some(value.clone()),
            Self::Style(style) => styles.get(*style),
            Self::Computed(derived) => Some(derived(styles)),
        }
    }

    /// Get the value, or a style from the styles.
    #[inline(always)]
    pub fn get_or(&self, styles: &Styles, default: T) -> T
    where
        T: Clone + 'static,
    {
        self.get(styles).unwrap_or(default)
    }

    /// Get the value, or a style from the styles.
    #[inline(always)]
    pub fn get_or_else<F>(&self, styles: &Styles, default: F) -> T
    where
        T: Clone + 'static,
        F: FnOnce() -> T,
    {
        self.get(styles).unwrap_or_else(default)
    }
}

impl<T: Debug> Debug for Styled<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(value) => write!(f, "Styled::Value({:?})", value),
            Self::Style(style) => write!(f, "Styled::Style({:?})", style.key),
            Self::Computed(_) => write!(f, "Styled::Computed(...)"),
        }
    }
}

impl<T> From<T> for Styled<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

impl<T> From<Style<T>> for Styled<T> {
    fn from(style: Style<T>) -> Self {
        Self::Style(style)
    }
}

/// Hash a style key.
///
/// This uses the FNV-1a hash algorithm, with a 64-bit seed.
#[inline(always)]
pub const fn hash_style_key(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325;

    let mut index = 0;
    while index < bytes.len() {
        hash ^= bytes[index] as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        index += 1;
    }

    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY_A: Style<u32> = Style::new("a");
    const KEY_B: Style<u32> = Style::new("b");

    #[test]
    fn style_value() {
        let mut styles = Styles::new();

        assert_eq!(
            styles.get(KEY_A),
            None,
            "style should not exist before insertion"
        );

        styles.insert_value(KEY_A, 42);

        assert_eq!(styles.get(KEY_A), Some(42));
    }

    #[test]
    fn style_key() {
        let mut styles = Styles::new();

        assert_eq!(
            styles.get(KEY_A),
            None,
            "style should not exist before insertion"
        );

        styles.insert_style(KEY_A, KEY_B);

        assert_eq!(
            styles.get(KEY_A),
            None,
            "style should not exist before insertion"
        );

        styles.insert_value(KEY_B, 42);

        assert_eq!(styles.get(KEY_A), Some(42));
        assert_eq!(styles.get(KEY_B), Some(42));
    }
}
