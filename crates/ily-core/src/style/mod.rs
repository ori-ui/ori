mod loader;
mod parse;
mod selector;
mod style;
mod transition;

pub use loader::*;
pub use selector::*;
pub use style::*;
pub use transition::*;

#[macro_export]
macro_rules! attributes {
    {
        $context:expr, $self:expr
        $(, $field:ident: $name:literal $(($trans:expr))?)* $(,)?
    } => {
        $(attributes!(@internal $context, $self, $field: $name $(($trans))?);)*
    };
    {
        @internal $context:expr, $self:expr,
        $field:ident: $name:literal
    } => {
        let $field = {
            let attr = $context.style_value($name);
            attr.unwrap_or_else(|| {
                $self.$field.clone().unwrap_or_default()
            })
        };
    };
    {
        @internal $context:expr, $self:expr,
        $field:ident: $name:literal ($trans:expr)
    } => {
        let $field = {
            let attr = $context.style_value($name);
            match attr {
                Some((value, transition)) => {
                    let (value, should_redraw) = $trans.update(value, transition, $context.state.delta());

                    if should_redraw {
                        $context.request_redraw();
                    }

                    value
                },
                None => {
                    $self.$field.clone().unwrap_or_default()
                }
            }
        };
    };
}
