use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    hash::{BuildHasherDefault, Hasher},
    marker::PhantomData,
    sync::Arc,
};

/// Create a collection of styles.
#[macro_export]
macro_rules! style {
    ($styles:expr, $prefix:expr, $key:literal : $style:expr) => {
        $styles.insert_value(::std::concat!($prefix, $key), $style);
    };
    ($styles:expr, $prefix:expr, $key:literal -> $style:expr) => {
        $styles.insert_style(::std::concat!($prefix, $key), $style);
    };
    ($styles:expr, $prefix:expr, $key:literal : $style:expr, $($rest:tt)*) => {
        $crate::style! { $styles, $prefix, $key : $style }
        $crate::style! { $styles, $prefix, $($rest)* }
    };
    ($styles:expr, $prefix:expr, $key:literal -> $style:expr, $($rest:tt)*) => {
        $crate::style! { $styles, $prefix, $key -> $style }
        $crate::style! { $styles, $prefix, $($rest)* }
    };
    ($styles:expr, $prefix:expr, $key:literal { $($inner:tt)* }) => {
        $crate::style! { $styles, ::std::concat!($prefix, $key, "."), $($inner)* }
    };
    ($styles:expr, $prefix:expr, $key:literal { $($inner:tt)* }, $($rest:tt)*) => {
        $crate::style! { $styles, ::std::concat!($prefix, $key, "."), $($inner)* }
        $crate::style! { $styles, $prefix, $($rest)* }
    };
    ($styles:expr, $prefix:expr, ) => {};
    ($styles:expr, $key:literal $($tt:tt)*) => {
        $crate::style! { $styles, "", $key $($tt)* }
    };
    ($key:literal $($tt:tt)*) => {{
        #[allow(unused_mut)]
        let mut styles = $crate::style::Styles::new();
        $crate::style! { styles, $key $($tt)* }
        styles
    }};
    () => {
        $crate::style::Styles::new()
    };
}

#[derive(Clone)]
enum StyleEntry {
    Value(TypeId, Arc<dyn Any>),
    Key(u64),
}

#[derive(Clone, Default)]
struct StylesHasher(u64);

impl Hasher for StylesHasher {
    fn write(&mut self, _bytes: &[u8]) {
        unreachable!()
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }

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

    /// Insert a styled value.
    pub fn insert<T: 'static>(&mut self, key: &str, styled: Styled<T>) {
        match styled {
            Styled::Value(value) => self.insert_value(key, value),
            Styled::Style(style) => {
                let key = hash_style_key(key.as_bytes());
                let value = StyleEntry::Key(style.key);
                Arc::make_mut(&mut self.styles).insert(key, value);
            }
            Styled::Computed(derived) => self.insert_value(key, derived(self)),
        }
    }

    /// Insert a style.
    pub fn insert_value<T: 'static>(&mut self, key: &str, style: T) {
        let key = hash_style_key(key.as_bytes());
        let entry = StyleEntry::Value(TypeId::of::<T>(), Arc::new(style));
        Arc::make_mut(&mut self.styles).insert(key, entry);
    }

    /// Insert a style key.
    pub fn insert_style(&mut self, key: &str, style: &str) {
        self.insert_style_keys(
            hash_style_key(key.as_bytes()),
            hash_style_key(style.as_bytes()),
        );
    }

    /// Insert a style key.
    pub fn insert_style_keys(&mut self, key: u64, style: u64) {
        Arc::make_mut(&mut self.styles).insert(key, StyleEntry::Key(style));
    }

    /// Insert a style key.
    pub fn with<T: 'static>(mut self, key: &str, styled: Styled<T>) -> Self {
        self.insert(key, styled);
        self
    }

    /// Insert a styled value.
    pub fn with_value<T: 'static>(mut self, key: &str, value: T) -> Self {
        self.insert_value(key, value);
        self
    }

    /// Insert a style key.
    pub fn with_style(mut self, key: &str, style: &str) -> Self {
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
const fn hash_style_key(bytes: &[u8]) -> u64 {
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
    use crate::{canvas::Color, style::Style};

    #[test]
    fn style_macro() {
        let styles = style! {
            "primary": Color::BLUE,

            "button" {
                "color" -> "primary",
            },
        };

        assert_eq!(styles.get(Style::new("button.color")), Some(Color::BLUE));
    }
}
