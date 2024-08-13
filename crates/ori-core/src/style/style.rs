use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    hash::BuildHasherDefault,
    mem::{self, ManuallyDrop},
    sync::Arc,
};

use super::Palette;

/// Get a value from the current style.
#[track_caller]
pub fn style<T: Clone + Style + Any>() -> T {
    Styles::context(|style| style.get())
}

/// Get a value from the current style or a default value.
#[track_caller]
pub fn style_or<T: Clone + Any>(default: T) -> T {
    try_style().unwrap_or(default)
}

/// Try getting a value from the current style.
#[track_caller]
pub fn try_style<T: Clone + Any>() -> Option<T> {
    Styles::context(|style| style.try_get())
}

/// Run a closure with the given style.
pub fn styled<T>(style: impl Any, f: impl FnOnce() -> T) -> T {
    let mut new_style = Styles::snapshot();
    new_style.set(style);
    new_style.as_context(f)
}

type Builder<T> = Box<dyn Fn(&Styles) -> T + 'static>;

#[derive(Clone)]
struct Entry {
    value: Arc<dyn Any>,
    is_builder: bool,
}

impl Entry {
    unsafe fn value<T: Any>(&self) -> &T {
        unsafe { &*Arc::as_ptr(&self.value).cast::<T>() }
    }

    unsafe fn builder<T: Any>(&self) -> &Builder<T> {
        unsafe { &*Arc::as_ptr(&self.value).cast::<Builder<T>>() }
    }
}

/// A map of style values.
#[derive(Clone, Default)]
pub struct Styles {
    items: Arc<HashMap<TypeId, Entry, BuildHasherDefault<seahash::SeaHasher>>>,
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
    pub fn set<T: Any>(&mut self, item: T) {
        // now this is all sorts of cursed but this is the only way to do this
        if TypeId::of::<T>() == TypeId::of::<Self>() {
            let styles = ManuallyDrop::new(item);

            unsafe {
                // SAFETY: we know that the type is Style
                self.extend(mem::transmute_copy(&styles));
            }

            return;
        }

        let entry = Entry {
            value: Arc::new(item),
            is_builder: false,
        };

        Arc::make_mut(&mut self.items).insert(TypeId::of::<T>(), entry);
    }

    /// Build a value in a style.
    ///
    /// This is useful for when the value is dependent on other values in the style, like a [`Palette`].
    pub fn builder<T: Any>(&mut self, builder: impl Fn(&Styles) -> T + 'static) {
        let builder: Builder<T> = Box::new(builder);

        let entry = Entry {
            value: Arc::new(builder),
            is_builder: true,
        };

        Arc::make_mut(&mut self.items).insert(TypeId::of::<T>(), entry);
    }

    /// Set a value in a style returning the style.
    pub fn with(mut self, item: impl Any) -> Self {
        self.set(item);
        self
    }

    /// Set a value in a style returning the style.
    ///
    /// This is useful for when the value is dependent on other values in the style, like a [`Palette`].
    pub fn build<T: Any>(mut self, builder: impl Fn(&Styles) -> T + 'static) -> Self {
        self.builder(builder);
        self
    }

    /// Get a value from the current style.
    pub fn get<T: Clone + Style + Any>(&self) -> T {
        match self.try_get::<T>() {
            Some(value) => value,
            None => T::style(self),
        }
    }

    /// Try getting a value from the current style.
    pub fn try_get<T: Clone + Any>(&self) -> Option<T> {
        let value = self.items.get(&TypeId::of::<T>())?;

        if value.is_builder {
            unsafe { Some(value.builder::<T>()(self)) }
        } else {
            unsafe { Some(value.value::<T>().clone()) }
        }
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
    #[track_caller]
    pub fn context<T>(f: impl FnOnce(&mut Self) -> T) -> T {
        let result = Self::CONTEXT.with(|styles| {
            // call the function with the current style
            styles.try_borrow_mut().map(|mut styles| f(&mut styles))
        });

        match result {
            Ok(result) => result,
            Err(_) => panic!(
                "Styles context not accessable. Are you perhaps calling `style`, `style_or`, `try_style` or `palette` in a `Style` impl?"
            ),
        }
    }

    /// Get a snapshot of the current style.
    #[track_caller]
    pub fn snapshot() -> Self {
        Self::context(|style| style.clone())
    }
}

/// A trait for styling a value.
///
/// This is implemented for all types that are `Default`.
pub trait Style: Sized {
    /// Style a value.
    fn style(style: &Styles) -> Self;
}

impl<T: Default> Style for T {
    fn style(_: &Styles) -> Self {
        T::default()
    }
}
