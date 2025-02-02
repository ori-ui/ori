use std::{borrow::Cow, fmt::Debug, marker::PhantomData, mem, sync::Arc};

use super::Styles;

pub use ori_macro::Stylable;

/// A stylable type.
pub trait Stylable {
    /// The style type.
    type Style;

    /// Get the style.
    fn style(&self, styles: &Styles) -> Self::Style;
}

/// A style.
#[repr(C)]
pub struct Style<T: ?Sized> {
    pub(crate) key: Cow<'static, str>,
    pub(crate) hash: u64,
    pub(crate) marker: PhantomData<fn(&T)>,
}

impl<T: ?Sized> Style<T> {
    /// Create a new style.
    #[inline(always)]
    pub const fn new(key: &'static str) -> Self {
        Self {
            hash: hash_style_key(key.as_bytes()),
            key: Cow::Borrowed(key),
            marker: PhantomData,
        }
    }

    /// Create a new style from a string.
    #[inline(always)]
    pub fn from_string(key: String) -> Self {
        Self {
            hash: hash_style_key(key.as_bytes()),
            key: Cow::Owned(key),
            marker: PhantomData,
        }
    }

    #[inline(always)]
    pub(crate) fn cast<U: ?Sized>(&self) -> &Style<U> {
        // SAFETY: `T` is only present in `PhantomData`, and `Style` is `repr(C)`,
        //         so the layout is always the same, hence this is safe.
        unsafe { mem::transmute(self) }
    }
}

impl<T: ?Sized> Clone for Style<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            hash: self.hash,
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Debug for Style<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Style").field("key", &self.key).finish()
    }
}

impl<T: ?Sized> PartialEq for Style<T> {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl<T: ?Sized> Eq for Style<T> {}

impl<T: ?Sized> From<&'static str> for Style<T> {
    fn from(key: &'static str) -> Self {
        Self::new(key)
    }
}

impl<T: ?Sized> From<String> for Style<T> {
    fn from(key: String) -> Self {
        Self::from_string(key)
    }
}

/// Create a style value, shorthand for [`Styled::new`].
#[inline(always)]
pub fn val<T>(val: impl Into<T>) -> Styled<T> {
    Styled::new(val.into())
}

/// Create a style key, shorthand for [`Styled::style`].
#[inline(always)]
pub fn style<T>(key: impl Into<Style<T>>) -> Styled<T> {
    Styled::style(key.into())
}

/// Create a computed style, shorthand for [`Styled::computed`].
///
/// **Note:** This is by far the least efficient variant, and should only be used when necessary.
#[inline(always)]
pub fn comp<T>(f: impl Fn(&Styles) -> T + Send + Sync + 'static) -> Styled<T> {
    Styled::computed(f)
}

/// A computed style.
///
// Box<dyn Fn()> is 16 bytes large, however Arc<Box<dyn Fn()>> is only 8 bytes. since computed
// styles are used so infrequently, compared to the other variants, it's worth the tradeoff to save
// memory, even if it costs an extra indirection.
pub type Computed<T> = Arc<Box<dyn Fn(&Styles) -> T + Send + Sync>>;

#[derive(Clone)]
pub(crate) enum StyledInner<T> {
    /// A value.
    Value(T),

    /// A style key.
    Style(Style<T>),

    /// A derived style.
    Computed(Computed<T>),
}

/// A styled value.
#[derive(Clone)]
pub struct Styled<T> {
    pub(crate) inner: StyledInner<T>,
}

impl<T> Styled<T> {
    /// Create a new styled value.
    #[inline(always)]
    pub fn new(style: impl Into<Styled<T>>) -> Self {
        style.into()
    }

    /// Create a new styled from a value.
    pub fn value(val: T) -> Self {
        Self {
            inner: StyledInner::Value(val),
        }
    }

    /// Create a new styled from a style key.
    #[inline(always)]
    pub fn style(key: impl Into<Style<T>>) -> Self {
        Self {
            inner: StyledInner::Style(key.into()),
        }
    }

    /// Create a new computed styled value.
    ///
    /// **Note:** This is by far the least efficient variant, and should only be used when
    /// necessary.
    #[inline(always)]
    pub fn computed(f: impl Fn(&Styles) -> T + Send + Sync + 'static) -> Self {
        Self {
            inner: StyledInner::Computed(Arc::new(Box::new(f))),
        }
    }

    /// Get the value, or a style from the styles.
    #[inline(always)]
    pub fn get(&self, styles: &Styles) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        match self.inner {
            StyledInner::Value(ref value) => Some(value.clone()),
            StyledInner::Style(ref style) => styles.get(style),
            StyledInner::Computed(ref comp) => Some(comp(styles)),
        }
    }

    /// Get the value, or a style from the styles.
    #[inline(always)]
    pub fn get_or(&self, styles: &Styles, default: T) -> T
    where
        T: Clone + Send + Sync + 'static,
    {
        self.get(styles).unwrap_or(default)
    }

    /// Get the value, or a style from the styles.
    #[inline(always)]
    pub fn get_or_else<F>(&self, styles: &Styles, default: F) -> T
    where
        T: Clone + Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        self.get(styles).unwrap_or_else(default)
    }
}

impl<T: Debug> Debug for Styled<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.inner {
            StyledInner::Value(ref value) => f.debug_tuple("Styled::Value").field(value).finish(),
            StyledInner::Style(ref style) => f.debug_tuple("Styled::Style").field(style).finish(),
            StyledInner::Computed(..) => f.debug_tuple("Styled::Computed").finish(),
        }
    }
}

impl<T> From<T> for Styled<T> {
    fn from(value: T) -> Self {
        Self {
            inner: StyledInner::Value(value),
        }
    }
}

impl<T> From<Style<T>> for Styled<T> {
    fn from(style: Style<T>) -> Self {
        Self {
            inner: StyledInner::Style(style),
        }
    }
}

/// Hash a style key.
///
/// This uses the FNV-1a hash algorithm.
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

/// Hash a style key.
///
/// This uses the FNV-1a hash algorithm.
#[inline(always)]
pub const fn hash_style_key_u64(bytes: &[u64]) -> u64 {
    let mut hash = 0xcbf29ce484222325;

    let mut index = 0;
    while index < bytes.len() {
        hash ^= bytes[index];
        hash = hash.wrapping_mul(0x100000001b3);
        index += 1;
    }

    hash
}
