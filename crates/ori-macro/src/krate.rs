use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::FoundCrate;
use quote::quote;

fn found_crate(krate: FoundCrate) -> TokenStream {
    match krate {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(#ident)
        }
    }
}

pub fn find_crate(krate: &str) -> TokenStream {
    let ident = Ident::new(krate, Span::call_site());
    let krate = format!("ori-{}", krate);

    match proc_macro_crate::crate_name(&krate) {
        Ok(krate) => found_crate(krate),
        Err(_) => match proc_macro_crate::crate_name("ori") {
            Ok(krate) => {
                let krate = found_crate(krate);
                quote!(#krate::#ident)
            }
            Err(_) => quote!(ori::#ident),
        },
    }
}
