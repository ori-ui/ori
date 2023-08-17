mod builtin;
mod key;
mod palette;
mod theme;

pub use builtin::*;
pub use key::*;
pub use palette::*;
pub use theme::*;

#[macro_export]
macro_rules! style {
    ($module_vis:vis $module:ident {
        $($vis:vis const $name:ident : $ty:ty = $expr:expr;)*
    }) => {
        $module_vis mod $module {
            use super::*;

            $(pub const $name: $crate::Key<$ty> = $crate::Key::new(
                ::std::concat!(::std::stringify!($module), ".", ::std::stringify!($name))
            );)*

            $module_vis fn default_theme() -> $crate::Theme {
                $crate::Theme::new()
                    $(.with($name, $expr))*
            }
        }
    };
}
