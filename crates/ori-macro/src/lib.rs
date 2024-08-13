//! Macros for the Ori crate.

#![warn(missing_docs)]

mod build;
mod entry;
mod example;
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

#[allow(dead_code)]
fn find_winit() -> syn::Path {
    match proc_macro_crate::crate_name("ori-winit") {
        Ok(krate) => found_crate(krate),
        Err(_) => match proc_macro_crate::crate_name("ori") {
            Ok(krate) => {
                let ori = found_crate(krate);
                syn::parse_quote!(#ori::winit)
            }
            Err(_) => syn::parse_quote!(ori::winit),
        },
    }
}

#[doc(hidden)]
#[proc_macro]
pub fn lowercase_ident(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let ident = syn::parse2::<syn::Ident>(input).unwrap();
    let ident = ident.to_string().to_lowercase();
    quote::quote!(#ident).into()
}

#[manyhow::manyhow]
#[proc_macro]
/// Load a font from a file or directory.
///
/// The path is relative to the `Cargo.toml` file.
pub fn include_font(input: proc_macro::TokenStream) -> manyhow::Result<proc_macro::TokenStream> {
    font::include_font(input)
}

/// Derive the `Rebuild` trait.
#[manyhow::manyhow]
#[proc_macro_derive(Rebuild, attributes(rebuild))]
pub fn derive_rebuild(input: proc_macro::TokenStream) -> manyhow::Result<proc_macro::TokenStream> {
    rebuild::derive_rebuild(input)
}

/// Derive the builder pattern.
#[manyhow::manyhow]
#[proc_macro_derive(Build, attributes(build))]
pub fn derive_build(input: proc_macro::TokenStream) -> manyhow::Result<proc_macro::TokenStream> {
    build::derive_build(input)
}

/// Only include the annotated item on desktop platforms.
#[proc_macro_attribute]
pub fn desktop(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let expanded = quote::quote! {
        #[cfg(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "linux",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "netbsd",
        ))]
        #input
    };

    expanded.into()
}

/// Only include the annotated item on mobile platforms.
#[proc_macro_attribute]
pub fn mobile(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let expanded = quote::quote! {
        #[cfg(any(
            target_os = "android",
            target_os = "ios",
        ))]
        #input
    };

    expanded.into()
}

/// Embed an example in the documentation.
///
/// This is an internal macro used by the `ori` crate.
#[manyhow::manyhow]
#[proc_macro_attribute]
pub fn example(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> manyhow::Result<proc_macro::TokenStream> {
    example::example(args, input)
}

/// A macro to generate boilerplate for the `main` function.
///
/// This is useful when targeting mobile platforms.
#[manyhow::manyhow]
#[proc_macro_attribute]
pub fn main(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> manyhow::Result<proc_macro::TokenStream> {
    entry::main(args, input)
}
