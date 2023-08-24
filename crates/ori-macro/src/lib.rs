//! Macros for the Ori crate.

#![warn(missing_docs)]

mod font;
mod rebuild;

fn found_crate(krate: proc_macro_crate::FoundCrate) -> syn::Path {
    match krate {
        proc_macro_crate::FoundCrate::Itself => syn::parse_quote!(crate),
        proc_macro_crate::FoundCrate::Name(name) => {
            let ident = proc_macro2::Ident::new(&name, proc_macro2::Span::call_site());
            syn::parse_quote!(::#ident)
        }
    }
}

fn find_core() -> syn::Path {
    match proc_macro_crate::crate_name("ori-core") {
        Ok(krate) => found_crate(krate),
        Err(_) => match proc_macro_crate::crate_name("ori") {
            Ok(krate) => {
                let ori = found_crate(krate);
                syn::parse_quote!(#ori::core)
            }
            Err(_) => syn::parse_quote!(ori::core),
        },
    }
}

#[manyhow::manyhow]
#[proc_macro]
/// Load a font from a file or directory.
///
/// The path is relative to the `Cargo.toml` file.
pub fn font(input: proc_macro::TokenStream) -> manyhow::Result<proc_macro::TokenStream> {
    font::font(input)
}

/// Derive the `Rebuild` trait.
#[manyhow::manyhow]
#[proc_macro_derive(Rebuild, attributes(rebuild))]
pub fn derive_rebuild(input: proc_macro::TokenStream) -> manyhow::Result<proc_macro::TokenStream> {
    rebuild::derive_rebuild(input)
}
