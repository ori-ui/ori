mod build;
mod krate;
mod view;

#[proc_macro_error::proc_macro_error]
#[proc_macro]
pub fn view(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    view::view(input)
}

#[proc_macro_error::proc_macro_error]
#[proc_macro_derive(Build, attributes(prop, event, bind, children))]
pub fn derive_build(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    build::derive_build(input)
}
