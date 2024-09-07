use std::{
    any::Any,
    collections::HashMap,
    fmt::Debug,
    hash::{BuildHasherDefault, Hasher},
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
    Value(Arc<dyn Any>),
    Key(u64),
}

#[derive(Clone, Default)]
struct StylesHasher(u64);

impl Hasher for StylesHasher {
    fn write(&mut self, bytes: &[u8]) {
        self.0 = seahash::hash(bytes);
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }

    fn finish(&self) -> u64 {
        self.0
    }
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
            Styled::Key(style) => {
                let key = seahash::hash(key.as_bytes());
                Arc::make_mut(&mut self.styles).insert(key, StyleEntry::Key(style));
            }
            Styled::Computed(derived) => self.insert_value(key, derived(self)),
        }
    }

    /// Insert a style.
    pub fn insert_value<T: 'static>(&mut self, key: &str, style: T) {
        let key = seahash::hash(key.as_bytes());
        Arc::make_mut(&mut self.styles).insert(key, StyleEntry::Value(Arc::new(style)));
    }

    /// Insert a style key.
    pub fn insert_style(&mut self, key: &str, style: &str) {
        let key = seahash::hash(key.as_bytes());
        let style = seahash::hash(style.as_bytes());
        Arc::make_mut(&mut self.styles).insert(key, StyleEntry::Key(style));
    }

    /// Extend the styles with another collection of styles.
    pub fn extend(&mut self, styles: impl Into<Styles>) {
        let styles = Arc::unwrap_or_clone(styles.into().styles);
        Arc::make_mut(&mut self.styles).extend(styles);
    }

    fn get_ref(&self, key: u64) -> Option<&dyn Any> {
        let style = self.styles.get(&key)?;

        match style {
            StyleEntry::Value(value) => Some(value.as_ref()),
            StyleEntry::Key(key) => self.get_ref(*key),
        }
    }

    /// Get a style.
    #[track_caller]
    pub fn get_keyed<T>(&self, key: u64) -> Option<T>
    where
        T: Clone + 'static,
    {
        let style = self.get_ref(key)?.downcast_ref::<T>();
        Some(style.expect("style is of type `T`").clone())
    }

    /// Get a style, or a default value.
    #[track_caller]
    pub fn get_keyed_or<T>(&self, default: T, key: u64) -> T
    where
        T: Clone + 'static,
    {
        self.get_keyed(key).unwrap_or(default)
    }

    /// Get a style, or a default value.
    #[track_caller]
    pub fn get_keyed_or_else<T, F>(&self, default: F, key: u64) -> T
    where
        T: Clone + 'static,
        F: FnOnce() -> T,
    {
        self.get_keyed(key).unwrap_or_else(default)
    }

    /// Get a style.
    #[track_caller]
    pub fn get<T>(&self, key: &str) -> Option<T>
    where
        T: Clone + 'static,
    {
        let key = seahash::hash(key.as_bytes());
        self.get_keyed(key)
    }

    /// Get a style, or a default value.
    #[track_caller]
    pub fn get_or<T>(&self, default: T, key: &str) -> T
    where
        T: Clone + 'static,
    {
        self.get(key).unwrap_or(default)
    }

    /// Get a style, or a default value.
    #[track_caller]
    pub fn get_or_else<T, F>(&self, default: F, key: &str) -> T
    where
        T: Clone + 'static,
        F: FnOnce() -> T,
    {
        self.get(key).unwrap_or_else(default)
    }
}

/// Create a style value.
pub fn val<T>(val: impl Into<T>) -> Styled<T> {
    Styled::Value(val.into())
}

/// Create a style key.
pub fn key<T>(key: &str) -> Styled<T> {
    Styled::key(key)
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
    Key(u64),

    /// A derived style.
    Computed(Computed<T>),
}

impl<T> Styled<T> {
    /// Create a new styled value, from a style key.
    pub fn key(key: &str) -> Self {
        Self::Key(seahash::hash(key.as_bytes()))
    }

    /// Get the value, or a style from the styles.
    pub fn get(&self, styles: &Styles) -> Option<T>
    where
        T: Clone + 'static,
    {
        match self {
            Self::Value(value) => Some(value.clone()),
            Self::Key(key) => styles.get_keyed::<T>(*key),
            Self::Computed(derived) => Some(derived(styles)),
        }
    }

    /// Get the value, or a style from the styles.
    pub fn get_or(&self, styles: &Styles, default: T) -> T
    where
        T: Clone + 'static,
    {
        self.get(styles).unwrap_or(default)
    }

    /// Get the value, or a style from the styles.
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
            Self::Key(key) => write!(f, "Styled::Key({:?})", key),
            Self::Computed(_) => write!(f, "Styled::Computed(...)"),
        }
    }
}

impl<T> From<T> for Styled<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

#[cfg(test)]
mod tests {

    use crate::canvas::Color;

    #[test]
    fn style_macro() {
        let styles = style! {
            "primary": Color::BLUE,

            "button" {
                "color" -> "primary",
            },
        };

        assert_eq!(styles.get("button.color"), Some(Color::BLUE));
    }
}
