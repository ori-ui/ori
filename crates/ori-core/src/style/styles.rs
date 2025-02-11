use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::{self, Debug},
    hash::BuildHasherDefault,
};

use seahash::SeaHasher;

use crate::{context::RebuildCx, rebuild::Rebuild};

/// A trait implemented by styles.
///
/// This will allow the style to be used in the arguments of [`StyleBuilder`] functions.
///
/// # Example
/// ```rust
/// # use ori_core::{
/// #    style::{Style, Styles, StyleBuilder, Theme},
/// #    views::ButtonStyle,
/// #    canvas::Color,
/// #    layout::Padding,
/// # };
/// struct MyStyle {
///     my_color: Color,
///     my_padding: Padding,
/// }
///
/// impl Style for MyStyle {
///     fn default_style() -> StyleBuilder<Self> {
///         StyleBuilder::new(|theme: &Theme, button: &ButtonStyle| MyStyle {
///             my_color: theme.accent,
///             my_padding: button.padding,
///         })
///     }
/// }
///
/// let mut styles = Styles::new()
///     .with(|| Theme {
///         accent: Color::rgb(1.0, 0.0, 0.0),
///         ..Theme::default()
///     })
///     .with(|theme: &Theme| ButtonStyle {
///         padding: Padding::all(10.0),
///         ..Default::default()
///     });
///
/// let my_style = styles.style::<MyStyle>();
///
/// assert_eq!(my_style.my_color, Color::rgb(1.0, 0.0, 0.0));
/// assert_eq!(my_style.my_padding, Padding::all(10.0));
/// ```
pub trait Style: Sized {
    /// The default style of the object.
    fn default_style() -> StyleBuilder<Self>;
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

/// A collection of [`Style`]s.
#[derive(Default)]
pub struct Styles {
    builders: HashMap<TypeId, StyleBuilder<Box<dyn Any>>, StylesHasher>,
    cache: HashMap<TypeId, Box<dyn Any>, StylesHasher>,
}

impl Debug for Styles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Styles")
            .field("builders", &self.builders)
            .finish()
    }
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
    /// See [`StyleBuilder::new`] for more information.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ori_core::{style::{Styles, Theme}, views::ButtonStyle};
    /// Styles::new().insert(|theme: &Theme| ButtonStyle {
    ///     color: theme.accent,
    ///     ..Default::default()
    /// });
    /// ```
    pub fn insert<T, B>(&mut self, builder: B) -> bool
    where
        B: IntoStyleBuilder<T> + 'static,
        B::Output: Style + Any,
    {
        let type_id = TypeId::of::<B::Output>();
        let dyn_builder = StyleBuilder::new(builder).into_dyn();

        if self.builders.contains_key(&type_id) {
            self.cache.clear();
        }

        self.builders.insert(type_id, dyn_builder).is_some()
    }

    /// Insert a style builder into the collection.
    ///
    /// See [`StyleBuilder::new`] for more information.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ori_core::{style::{Styles, Theme}, views::ButtonStyle};
    /// let styles = Styles::new().with(|theme: &Theme| ButtonStyle {
    ///    color: theme.accent,
    ///    ..Default::default()
    /// });
    /// ```
    pub fn with<T, B>(mut self, builder: B) -> Self
    where
        B: IntoStyleBuilder<T> + 'static,
        B::Output: Style + Any,
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

        let builder = T::default_style().into_dyn();
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
///
/// See the [`Style`] trait for an example.
pub struct StyleBuilder<T> {
    type_name: &'static str,
    builder: Box<dyn Fn(&mut Styles) -> T>,
    dependencies: Vec<TypeId>,
}

impl<T: Any> StyleBuilder<T> {
    /// Create a new style builder.
    ///
    /// This takes a function that takes any number of references to [`Style`]s and returns a style.
    /// Each [`Style`] referenced will be created if not already present in the [`Styles`] collection.
    /// If a style is dependent on this style, it will be rebuilt when this style is rebuilt.
    ///
    /// __Note:__ Cyclic dependencies are not allowed and will result in a panic.
    pub fn new<U, B>(builder: B) -> Self
    where
        B: IntoStyleBuilder<U, Output = T> + 'static,
    {
        Self {
            type_name: std::any::type_name::<T>(),
            dependencies: B::dependencies(&builder),
            builder: Box::new(move |styles| builder.build(styles)),
        }
    }

    fn into_dyn(self) -> StyleBuilder<Box<dyn Any>> {
        StyleBuilder {
            type_name: self.type_name,
            builder: Box::new(move |styles| Box::new((self.builder)(styles))),
            dependencies: self.dependencies,
        }
    }
}

impl<T> Debug for StyleBuilder<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StyleBuilder")
            .field("type_name", &self.type_name)
            .finish()
    }
}

/// A trait for converting a function into a style builder.
///
/// See [`StyleBuilder::new`] for more information.
pub trait IntoStyleBuilder<T> {
    /// The type of style built by the builder.
    type Output;

    /// Build the style.
    fn build(&self, styles: &mut Styles) -> Self::Output;

    /// Get the dependencies of the style builder.
    fn dependencies(&self) -> Vec<TypeId> {
        Vec::new()
    }
}

impl<T> IntoStyleBuilder<StyleBuilder<T>> for StyleBuilder<T> {
    type Output = T;

    fn build(&self, styles: &mut Styles) -> Self::Output {
        (self.builder)(styles)
    }

    fn dependencies(&self) -> Vec<TypeId> {
        self.dependencies.clone()
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

            fn dependencies(&self) -> Vec<TypeId> {
                vec![$(TypeId::of::<$ty>()),*]
            }
        }
    };
}

impl_style_builder!(A, B, C, D, E, F, G, H, I, J, K, L);
