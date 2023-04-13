mod loader;
mod parse;
mod selector;
mod style;

pub use loader::*;
pub use selector::*;
pub use style::*;

#[macro_export]
macro_rules! attributes {
    {
        $context:expr, $self:expr
        $(, $field:ident: $name:literal)* $(,)?
    } => {
        $(
            let $field = {
                let attr = $context.style_value($name);
                attr.unwrap_or_else(|| {
                    $self.$field.clone().unwrap_or_default()
                })
            };
        )*
    };
}
