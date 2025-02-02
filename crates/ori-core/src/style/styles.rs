use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    hash::{BuildHasherDefault, Hasher},
    marker::PhantomData,
    str::FromStr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use seahash::SeaHasher;

use crate::style::hash_style_key_u64;

use super::{hash_style_key, Style, Styled, StyledInner};

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

    /// The current version of the styles.
    version: AtomicU64,
}

impl Debug for Styles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Styles").finish()
    }
}

#[derive(Clone, Default)]
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
type StyleEntry = StyledInner<Arc<dyn Any + Send + Sync>>;
type StyleConverter = Arc<dyn Fn(Arc<dyn Any + Send + Sync>) -> Arc<dyn Any + Send + Sync>>;
type StyleKey = (u64, TypeId);
type CacheEntry = Option<Arc<dyn Any + Send + Sync>>;

impl Styles {
    fn next_version() -> u64 {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        NEXT.fetch_add(1, Ordering::Relaxed)
    }

    /// Create a new set of styles.
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            root: StyleSet::default(),
            converters: HashMap::default(),
            cache: Mutex::new(HashMap::default()),
            version: AtomicU64::new(Self::next_version()),
        }
    }

    /// Get the current version of the styles.
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Relaxed)
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
        self.push_class_hash(class);
    }

    /// Push a class hash onto the stack.
    pub fn push_class_hash(&mut self, class: u64) {
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
        let entry = match value.into().inner {
            StyledInner::Value(value) => {
                let value: Arc<dyn Any + Send + Sync> = Arc::new(value);
                Styled::value(value)
            }
            StyledInner::Style(style) => Styled::style(Style {
                hash: style.hash,
                key: style.key,
                marker: PhantomData,
            }),
            StyledInner::Computed(computed) => Styled::computed(move |styles| {
                Arc::new(computed(styles)) as Arc<dyn Any + Send + Sync>
            }),
        };

        self.insert_any(&style.key, entry);
    }

    pub(crate) fn insert_any(&mut self, key: &str, entry: Styled<Arc<dyn Any + Send + Sync>>) {
        let mut classes = key.split('.').map(str::as_bytes).map(hash_style_key);

        let last = classes.next_back().unwrap();

        let mut current = &mut self.root;

        for class in classes {
            current = current.classes.entry(class).or_default();
        }

        current.styles.insert(last, entry.inner);
        self.invalidate();
    }

    fn invalidate(&mut self) {
        let _ = self.cache.get_mut().map(HashMap::clear);
        self.version.store(Self::next_version(), Ordering::Relaxed);
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
        self.invalidate();
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
            StyledInner::Value(value) => self.convert(value),
            StyledInner::Style(style) => self.get(style.cast()),
            StyledInner::Computed(computed) => self.convert(&computed(self)),
        }
    }

    fn convert<T>(&self, value: &Arc<dyn Any + Send + Sync>) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
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
        let cache = match self.cache.lock() {
            Ok(cache) => cache.clone(),
            Err(_) => HashMap::default(),
        };

        let version = self.version.load(Ordering::Relaxed);

        Self {
            stack: self.stack.clone(),
            root: self.root.clone(),
            converters: self.converters.clone(),
            cache: Mutex::new(cache),
            version: AtomicU64::new(version),
        }
    }
}

impl PartialEq for Styles {
    fn eq(&self, other: &Self) -> bool {
        self.version.load(Ordering::Relaxed) == other.version.load(Ordering::Relaxed)
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
