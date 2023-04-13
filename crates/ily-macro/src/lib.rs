mod view;

#[proc_macro]
#[proc_macro_error::proc_macro_error]
pub fn view(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    view::view(input)
}

#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn component(
    _attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    input
}
