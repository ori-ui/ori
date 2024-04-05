use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    hash::BuildHasher,
    mem::{self, ManuallyDrop},
    sync::Arc,
};

use super::Palette;

/// Get a value from the current style.
pub fn style<T: Clone + Styled + Any>() -> T {
    Styles::context(|style| style.get())
}

/// Get a value from the current style or a default value.
pub fn style_or<T: Clone + Any>(default: T) -> T {
    try_style().unwrap_or(default)
}

/// Try getting a value from the current style.
pub fn try_style<T: Clone + Any>() -> Option<T> {
    Styles::context(|style| style.try_get())
}

/// Run a closure with the given style.
pub fn styled<T>(style: impl Style, f: impl FnOnce() -> T) -> T {
    let mut new_style = Styles::snapshot();
    new_style.extend(style.into_styles());

    new_style.as_context(f)
}

type Builder<T> = Box<dyn Fn(&Styles) -> T + 'static>;

/// A map of style values.
#[derive(Clone, Default)]
pub struct Styles {
    items: Arc<HashMap<TypeId, Arc<dyn Any>, StyleHasher>>,
}

impl Styles {
    thread_local! {
        static CONTEXT: RefCell<Styles> = RefCell::new(Styles::default());
    }

    /// Create a new style.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the palette of the style.
    pub fn palette(&self) -> Palette {
        self.get()
    }

    /// Set a value in a style.
    pub fn set(&mut self, item: impl Style) {
        self.extend(item.into_styles());
    }

    /// Build a value in a style.
    ///
    /// This is useful for when the value is dependent on other values in the style, like a [`Palette`].
    pub fn build<T: Any>(&mut self, builder: impl Fn(&Styles) -> T + 'static) {
        let builder: Builder<T> = Box::new(builder);
        Arc::make_mut(&mut self.items).insert(TypeId::of::<T>(), Arc::new(builder));
    }

    /// Set a value in a style returning the style.
    pub fn with(mut self, item: impl Style) -> Self {
        self.set(item);
        self
    }

    /// Set a value in a style returning the style.
    ///
    /// This is useful for when the value is dependent on other values in the style, like a [`Palette`].
    pub fn with_build<T: Any>(mut self, builder: impl Fn(&Styles) -> T + 'static) -> Self {
        self.build(builder);
        self
    }

    /// Get a value from the current style.
    pub fn get<T: Clone + Styled + Any>(&self) -> T {
        match self.try_get::<T>() {
            Some(value) => value,
            None => T::from_style(self),
        }
    }

    /// Try getting a value from the current style.
    pub fn try_get<T: Clone + Any>(&self) -> Option<T> {
        let value = self.items.get(&TypeId::of::<T>())?;

        if let Some(builder) = value.downcast_ref::<Builder<T>>() {
            return Some(builder(self));
        }

        value.downcast_ref::<T>().cloned()
    }

    /// Extend the style with another style.
    pub fn extend(&mut self, other: Self) {
        Arc::make_mut(&mut self.items).extend(other.items.as_ref().clone());
    }

    /// Run a function with the given style as the current style.
    pub fn as_context<T>(&mut self, f: impl FnOnce() -> T) -> T {
        Self::CONTEXT.with_borrow_mut(|context| mem::swap(context, self));
        let result = f();
        Self::CONTEXT.with_borrow_mut(|context| mem::swap(context, self));
        result
    }

    /// Get the current style.
    pub fn context<T>(f: impl FnOnce(&mut Self) -> T) -> T {
        Self::CONTEXT.with_borrow_mut(f)
    }

    /// Get a snapshot of the current style.
    pub fn snapshot() -> Self {
        Self::context(|style| style.clone())
    }
}

/// A trait for styling a value.
///
/// This is implemented for all types that are `Default`.
pub trait Styled: Sized {
    /// Style a value.
    fn from_style(style: &Styles) -> Self;
}

impl<T: Default> Styled for T {
    fn from_style(_: &Styles) -> Self {
        T::default()
    }
}

/// A trait for converting a value into a style.
///
/// Turns a value into a style by wrapping it in a `Styles` instance, if it isn't already a `Styles`.
///
/// This trait should not be implemented manually.
pub trait Style {
    /// Convert a value into a style.
    fn into_styles(self) -> Styles;
}

impl<T: Any> Style for T {
    fn into_styles(self) -> Styles {
        // now this is all sorts of cursed but this is the only way to do this
        if TypeId::of::<Self>() == TypeId::of::<Styles>() {
            let style = ManuallyDrop::new(self);

            // SAFETY: we know that the type is Style
            return unsafe { mem::transmute_copy(&style) };
        }

        let mut style = Styles::new();
        Arc::make_mut(&mut style.items).insert(TypeId::of::<Self>(), Arc::new(self));
        style
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
