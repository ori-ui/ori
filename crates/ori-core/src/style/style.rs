use std::{
    any::{Any, TypeId},
    borrow::Cow,
    collections::HashMap,
    fmt::Debug,
    hash::{BuildHasherDefault, Hasher},
    marker::PhantomData,
    mem,
    str::FromStr,
    sync::{Arc, Mutex},
};

use seahash::SeaHasher;

#[repr(transparent)]
#[derive(Clone, Copy)]
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

/// A collection of styles.
#[derive(Default)]
pub struct Styles {
    /// The stack of current classes.
    stack: Vec<u64>,

    /// The root style set.
    root: StyleSet,

    /// The set of style converters.
    converters: HashMap<(TypeId, TypeId), StyleConverter>,

    /// The style cache.
    cache: Mutex<HashMap<StyleKey, CacheEntry, BuildHasherDefault<SeaHasher>>>,
}

impl Debug for Styles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Styles").field("root", &self.root).finish()
    }
}

#[derive(Clone, Debug, Default)]
struct StyleSet {
    classes: HashMap<u64, StyleSet, BuildStyleHasher>,
    styles: HashMap<u64, StyleEntry, BuildStyleHasher>,
}

impl StyleSet {
    fn extend(&mut self, other: StyleSet) {
        for (key, value) in other.classes {
            self.classes.entry(key).or_default().extend(value);
        }

        self.styles.extend(other.styles);
    }
}

type BuildStyleHasher = BuildHasherDefault<StylesHasher>;
type StyleEntry = Styled<Arc<dyn Any + Send + Sync>>;
type StyleConverter = Arc<dyn Fn(Arc<dyn Any + Send + Sync>) -> Arc<dyn Any + Send + Sync>>;
type StyleKey = (u64, TypeId);
type CacheEntry = Option<Arc<dyn Any + Send + Sync>>;

impl Styles {
    /// Create a new set of styles.
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            root: StyleSet {
                classes: HashMap::default(),
                styles: HashMap::default(),
            },
            converters: HashMap::default(),
            cache: Mutex::new(HashMap::default()),
        }
    }

    /// Add a conversion function.
    pub fn add_conversion<T, U>(&mut self, f: impl Fn(T) -> U + 'static)
    where
        T: Clone + Send + Sync + 'static,
        U: Clone + Send + Sync + 'static,
    {
        let f = Arc::new(move |value: Arc<dyn Any + Send + Sync>| {
            let value = value.downcast::<T>().unwrap();
            let value = f(value.as_ref().clone());
            Arc::new(value) as Arc<dyn Any + Send + Sync>
        });

        let signature = (TypeId::of::<T>(), TypeId::of::<U>());
        self.converters.insert(signature, f);
    }

    /// Push a class onto the stack.
    pub fn push_class(&mut self, class: &str) {
        let class = hash_style_key(class.as_bytes());
        self.stack.push(class);
    }

    /// Pop a class from the stack.
    pub fn pop_class(&mut self) -> bool {
        self.stack.pop().is_some()
    }

    /// Run a closure within a context of a class.
    pub fn with_class<T>(&mut self, class: &str, f: impl FnOnce(&mut Self) -> T) -> T {
        let class = hash_style_key(class.as_bytes());

        self.stack.push(class);
        let result = f(self);
        self.stack.pop();

        result
    }

    /// Insert a style into the styles.
    pub fn insert<T>(&mut self, style: Style<T>, value: impl Into<Styled<T>>)
    where
        T: Clone + Send + Sync + 'static,
    {
        let entry: StyleEntry = match value.into() {
            Styled::Value(value) => Styled::Value(Arc::new(value)),
            Styled::Style(style) => Styled::Style(Style {
                hash: style.hash,
                key: style.key,
                marker: PhantomData,
            }),
            Styled::Computed(..) => todo!(),
        };

        self.insert_entry(&style.key, entry);
    }

    pub(crate) fn insert_entry(&mut self, key: &str, entry: StyleEntry) {
        let mut classes = key.split('.').map(str::as_bytes).map(hash_style_key);

        let last = classes.next_back().unwrap();

        let mut current = &mut self.root;

        for class in classes {
            current = current.classes.entry(class).or_default();
        }

        current.styles.insert(last, entry);
        let _ = self.cache.get_mut().map(HashMap::clear);
    }

    /// Insert a style into the styles.
    pub fn with<T>(mut self, style: Style<T>, value: impl Into<Styled<T>>) -> Self
    where
        T: Clone + Send + Sync + 'static,
    {
        self.insert(style, value);
        self
    }

    /// Extend the styles with another set of styles.
    pub fn extend(&mut self, other: impl Into<Styles>) {
        let other = other.into();

        self.root.extend(other.root);
        self.converters.extend(other.converters);
        let _ = self.cache.get_mut().map(HashMap::clear);
    }

    /// Get a value from the styles.
    #[inline(always)]
    pub fn get<T>(&self, style: &Style<T>) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let stack_hash = hash_style_key_u64(self.stack.as_slice());
        let key = (style.hash ^ stack_hash, TypeId::of::<T>());

        if let Some(entry) = self.cache.lock().unwrap().get(&key) {
            let entry = entry.as_ref()?;
            let style = entry.downcast_ref::<T>()?;

            return Some(style.clone());
        }

        tracing::trace!(
            key = %key.0,
            stack = ?self.stack,
            type = ?std::any::type_name::<T>(),
            "cache miss for {:?}",
            style.key
        );

        let classes = style
            .key
            .split('.')
            .map(str::as_bytes)
            .map(hash_style_key)
            .map(|class| (class, true));

        let classes = self
            .stack
            .iter()
            .map(|&class| (class, false))
            .chain(classes)
            .collect::<Vec<_>>();

        let Some(style) = self.get_inner::<T>(&classes) else {
            self.cache.lock().unwrap().insert(key, None);
            return None;
        };

        let cache_entry: CacheEntry = Some(Arc::new(style.clone()));
        self.cache.lock().unwrap().insert(key, cache_entry);

        Some(style)
    }

    fn get_inner<T>(&self, classes: &[(u64, bool)]) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let entry = Self::get_uncached(&self.root, classes.iter().copied())?;

        match entry {
            Styled::Value(value) => {
                if let Some(value) = value.downcast_ref::<T>() {
                    return Some(value.clone());
                }

                let signature = (value.as_ref().type_id(), TypeId::of::<T>());

                match self.converters.get(&signature) {
                    Some(converter) => {
                        let value = converter(value.clone());
                        let value = value.downcast_ref::<T>().unwrap();
                        Some(value.clone())
                    }
                    None => {
                        tracing::error!(
                            "style could not be converted to '{}'",
                            std::any::type_name::<T>()
                        );

                        None
                    }
                }
            }
            Styled::Style(style) => self.get(style.cast()),
            Styled::Computed(..) => todo!(),
        }
    }

    /// Get a value from the styles.
    #[inline(always)]
    pub fn get_or<T>(&self, default: T, style: &Style<T>) -> T
    where
        T: Clone + Send + Sync + 'static,
    {
        self.get(style).unwrap_or(default)
    }

    /// Get a value from the styles.
    #[inline(always)]
    pub fn get_or_else<T, F>(&self, default: F, style: &Style<T>) -> T
    where
        T: Clone + Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        self.get(style).unwrap_or_else(default)
    }

    fn get_uncached(
        style_set: &StyleSet,
        mut classes: impl ExactSizeIterator<Item = (u64, bool)> + Clone,
    ) -> Option<&StyleEntry> {
        let (class, required) = classes.next()?;

        if classes.len() == 0 {
            return style_set.styles.get(&class);
        }

        if let Some(next_set) = style_set.classes.get(&class) {
            if let Some(entry) = Self::get_uncached(next_set, classes.clone()) {
                return Some(entry);
            }
        }

        if required {
            return None;
        }

        Self::get_uncached(style_set, classes)
    }
}

impl Clone for Styles {
    fn clone(&self) -> Self {
        Self {
            stack: self.stack.clone(),
            root: self.root.clone(),
            converters: self.converters.clone(),
            cache: Mutex::new(HashMap::default()),
        }
    }
}

impl From<&str> for Styles {
    fn from(s: &str) -> Self {
        match Styles::from_str(s) {
            Ok(styles) => styles,
            Err(err) => {
                tracing::error!("failed to parse styles: {}", err);
                Styles::new()
            }
        }
    }
}

/// A style.
pub struct Style<T: ?Sized> {
    key: Cow<'static, str>,
    hash: u64,
    marker: PhantomData<fn(&T)>,
}

impl<T: ?Sized> Style<T> {
    /// Create a new style.
    #[inline(always)]
    pub const fn new(key: &'static str) -> Self {
        Self {
            key: Cow::Borrowed(key),
            hash: hash_style_key(key.as_bytes()),
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

    fn cast<U: ?Sized>(&self) -> &Style<U> {
        // SAFETY: the marker is used to ensure that the type is correct.
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

/// Create a style value.
pub fn val<T>(val: impl Into<T>) -> Styled<T> {
    Styled::Value(val.into())
}

/// Create a style key.
pub fn style<T>(key: impl Into<Style<T>>) -> Styled<T> {
    Styled::Style(key.into())
}

/// Create a computed style.
pub fn comp<T>(f: impl Fn(&Styles) -> T + Send + Sync + 'static) -> Styled<T> {
    Styled::Computed(Arc::new(Box::new(f)))
}

/// A computed style.
///
// Box<dyn Fn()> is 16 bytes large, however Arc<Box<dyn Fn()>> is only 8 bytes. since computed
// styles are used so infrequently, compared to the other variants, it's worth the tradeoff to save
// memory, even if it costs an extra indirection.
pub type Computed<T> = Arc<Box<dyn Fn(&Styles) -> T + Send + Sync>>;

/// A styled value.
#[derive(Clone)]
pub enum Styled<T> {
    /// A value.
    Value(T),

    /// A style key.
    Style(Style<T>),

    /// A derived style.
    ///
    /// **Note:** This is by far the least efficient variant, and should only be used when
    /// necessary.
    Computed(Computed<T>),
}

impl<T> Styled<T> {
    /// Create a new styled value.
    pub fn new(style: impl Into<Styled<T>>) -> Self {
        style.into()
    }

    /// Create a new styled from a style key.
    pub fn style(key: impl Into<Style<T>>) -> Self {
        Self::Style(key.into())
    }

    /// Get the value, or a style from the styles.
    #[inline(always)]
    pub fn get(&self, styles: &Styles) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        match self {
            Self::Value(value) => Some(value.clone()),
            Self::Style(style) => styles.get(style),
            Self::Computed(derived) => Some(derived(styles)),
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
        match self {
            Self::Value(value) => write!(f, "Styled::Value({:?})", value),
            Self::Style(style) => write!(f, "Styled::Style({:?})", style.key),
            Self::Computed(_) => write!(f, "Styled::Computed(..)"),
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

const fn hash_style_key_u64(bytes: &[u64]) -> u64 {
    let mut hash = 0xcbf29ce484222325;

    let mut index = 0;
    while index < bytes.len() {
        hash ^= bytes[index];
        hash = hash.wrapping_mul(0x100000001b3);
        index += 1;
    }

    hash
}
