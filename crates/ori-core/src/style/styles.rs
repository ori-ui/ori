use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::BuildHasherDefault,
};

use seahash::SeaHasher;

use crate::{context::RebuildCx, rebuild::Rebuild};

/// A trait for styles.
pub trait Style: Sized {
    /// The default style builder.
    fn builder() -> StyleBuilder<Self>;
}

/// A trait for stylable objects.
pub trait Stylable {
    /// The style type.
    type Style: Style;

    /// Style the object.
    ///
    /// This is done by creating a new style based on the given base style.
    fn style(&self, base: &Self::Style) -> Self::Style;

    /// Rebuild the style of the object.
    fn rebuild_style(&self, cx: &mut RebuildCx, style: &mut Self::Style)
    where
        Self::Style: Rebuild + 'static,
    {
        let new = self.style(cx.style());
        new.rebuild(cx, style);
        *style = new;
    }
}

/// A collection of styles.
#[derive(Default)]
pub struct Styles {
    builders: HashMap<TypeId, StyleBuilder<Box<dyn Any>>, StylesHasher>,
    cache: HashMap<TypeId, Box<dyn Any>, StylesHasher>,
}

type StylesHasher = BuildHasherDefault<SeaHasher>;

impl Styles {
    /// Create a new collection of styles.
    pub fn new() -> Styles {
        Styles::default()
    }

    /// Check if a style is contained in the collection.
    pub fn contains<T: Any>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.cache.contains_key(&type_id) || self.builders.contains_key(&type_id)
    }

    /// Insert a style builder into the collection.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ori_core::{style::{Styles, Theme}, views::ButtonStyle};
    /// Styles::new().insert(|theme: &Theme| ButtonStyle {
    ///     color: theme.accent,
    ///     ..todo!()
    /// });
    /// ```
    pub fn insert<T, B>(&mut self, builder: B) -> bool
    where
        B: IntoStyleBuilder<T> + 'static,
        B::Output: Any,
    {
        let type_id = TypeId::of::<B::Output>();

        let dyn_builder = StyleBuilder {
            builder: Box::new(move |styles| Box::new(builder.build(styles)) as Box<dyn Any>),
            dependencies: B::dependencies(),
        };

        if self.builders.contains_key(&type_id) {
            self.cache.clear();
        }

        self.builders.insert(type_id, dyn_builder).is_some()
    }

    /// Insert a style builder into the collection.
    ///
    /// See [`Styles::insert`] for more information.
    pub fn with<T, B>(mut self, builder: B) -> Self
    where
        B: IntoStyleBuilder<T> + 'static,
        B::Output: Any,
    {
        self.insert(builder);
        self
    }

    /// Extend the collection with another collection of styles.
    pub fn extend(&mut self, other: Styles) {
        for (type_id, builder) in other.builders {
            self.builders.insert(type_id, builder);
        }

        self.cache.clear();
    }

    /// Get a style from the collection.
    pub fn style<T: Style + Any>(&mut self) -> &T {
        if self.contains::<T>() {
            return self.get::<T>().unwrap();
        }

        let builder = T::builder().into_dyn();
        self.builders.insert(TypeId::of::<T>(), builder);
        self.get::<T>().unwrap()
    }

    /// Get a style from the collection.
    pub fn get<T: Any>(&mut self) -> Option<&T> {
        let type_id = TypeId::of::<T>();

        if self.cache.contains_key(&type_id) {
            return self.cache.get(&type_id)?.downcast_ref();
        }

        if let Some(builder) = self.builders.remove(&type_id) {
            let value = (builder.builder)(self);
            self.cache.insert(type_id, value);
            self.builders.insert(type_id, builder);
            return self.cache.get(&type_id)?.downcast_ref();
        }

        None
    }

    fn get_cached<T: Any>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.cache.get(&type_id)?.downcast_ref()
    }
}

/// A style builder.
pub struct StyleBuilder<T> {
    builder: Box<dyn Fn(&mut Styles) -> T>,
    dependencies: Vec<TypeId>,
}

impl<T: Any> StyleBuilder<T> {
    /// Create a new style builder.
    pub fn new<U, B>(builder: B) -> Self
    where
        B: IntoStyleBuilder<U, Output = T> + 'static,
    {
        Self {
            builder: Box::new(move |styles| builder.build(styles)),
            dependencies: B::dependencies(),
        }
    }

    fn into_dyn(self) -> StyleBuilder<Box<dyn Any>> {
        StyleBuilder {
            builder: Box::new(move |styles| Box::new((self.builder)(styles))),
            dependencies: self.dependencies,
        }
    }
}

/// A trait for converting a function into a style builder.
pub trait IntoStyleBuilder<T> {
    /// The output type.
    type Output;

    /// Build the style.
    fn build(&self, styles: &mut Styles) -> Self::Output;

    /// Get the dependencies of the style builder.
    fn dependencies() -> Vec<TypeId> {
        Vec::new()
    }
}

macro_rules! impl_style_builder {
    (@) => {};
    (@ $last:ident $(, $ty:ident)*) => {
        impl_style_builder!($($ty),*);
    };
    ($($ty:ident),*) => {
        impl_style_builder!(@ $($ty),*);

        impl<$($ty: Style + Any,)* FN, R> IntoStyleBuilder<fn($($ty,)*) -> R> for FN
        where
            FN: Fn($(&$ty,)*) -> R,
        {
            type Output = R;

            #[allow(non_snake_case, unused_variables)]
            fn build(&self, styles: &mut Styles) -> Self::Output {
                $(styles.style::<$ty>();)*
                $(let $ty = styles.get_cached::<$ty>().unwrap();)*
                (self)($($ty.clone(),)*)
            }

            fn dependencies() -> Vec<TypeId> {
                vec![$(TypeId::of::<$ty>()),*]
            }
        }
    };
}

impl_style_builder!(A, B, C, D, E, F, G, H, I, J, K, L);
