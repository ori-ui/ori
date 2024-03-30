use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    hash::BuildHasher,
    mem::{self, ManuallyDrop},
    sync::Arc,
};

/// Get a value from the current theme.
pub fn style<T: Clone + Default + Any>() -> T {
    Style::context(|theme| theme.get())
}

/// Get a value from the current theme or a default value.
pub fn style_or<T: Clone + Any>(default: T) -> T {
    Style::context(|theme| theme.try_get().cloned().unwrap_or(default))
}

/// Run a closure with the given style.
pub fn styled<T>(style: impl IntoStyle, f: impl FnOnce() -> T) -> T {
    let mut new_style = Style::snapshot();
    new_style.extend(style.into_style());

    new_style.as_context(f)
}

/// A map of style values.
#[derive(Clone, Default)]
pub struct Style {
    values: Arc<HashMap<TypeId, Arc<dyn Any>, StyleHasher>>,
}

impl Style {
    thread_local! {
        static CONTEXT: RefCell<Style> = RefCell::new(Style::default());
    }

    /// Create a new theme.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a value in a theme.
    pub fn set<T: Any>(&mut self, value: T) {
        Arc::make_mut(&mut self.values).insert(TypeId::of::<T>(), Arc::new(value));
    }

    /// Set a value in a theme returning the theme.
    pub fn with<T: Any>(mut self, value: T) -> Self {
        self.set(value);
        self
    }

    /// Get a value from the current theme.
    pub fn get<T: Clone + Default + Any>(&self) -> T {
        self.try_get().cloned().unwrap_or_default()
    }

    /// Try getting a value from the current theme.
    pub fn try_get<T: Any>(&self) -> Option<&T> {
        self.values.get(&TypeId::of::<T>())?.downcast_ref()
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

/// A trait for converting a value into a style.
///
/// Turns a value into a style by wrapping it in a `Style` instance, if it isn't already a `Style`.
///
/// This trait should not be implemented manually.
pub trait IntoStyle {
    /// Convert a value into a style.
    fn into_style(self) -> Style;
}

impl<T: Any> IntoStyle for T {
    fn into_style(self) -> Style {
        // now this is all sorts of cursed but this is the only way to do this
        if TypeId::of::<Self>() == TypeId::of::<Style>() {
            let style = ManuallyDrop::new(self);

            // SAFETY: we know that the type is Style
            return unsafe { mem::transmute_copy(&style) };
        }

        Style::default().with(self)
    }
}

#[derive(Clone, Default)]
struct StyleHasher;

impl BuildHasher for StyleHasher {
    type Hasher = seahash::SeaHasher;

    fn build_hasher(&self) -> Self::Hasher {
        seahash::SeaHasher::new()
    }
}
