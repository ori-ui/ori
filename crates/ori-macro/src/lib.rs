//! Macros for the Ori crate.

#![warn(missing_docs)]

mod build;
mod entry;
mod example;
mod font;
mod rebuild;

fn found_crate(krate: proc_macro_crate::FoundCrate, name: &str) -> syn::Path {
    match krate {
        proc_macro_crate::FoundCrate::Itself => {
            let ident = proc_macro2::Ident::new(name, proc_macro2::Span::call_site());
            syn::parse_quote!(::#ident)
        }
        proc_macro_crate::FoundCrate::Name(ref name) => {
            let ident = proc_macro2::Ident::new(name, proc_macro2::Span::call_site());
            syn::parse_quote!(::#ident)
        }
    }
}

fn find_core() -> syn::Path {
    match proc_macro_crate::crate_name("ori-core") {
        Ok(krate) => found_crate(krate, "ori_core"),
        Err(_) => match proc_macro_crate::crate_name("ori") {
            Ok(krate) => {
                let ori = found_crate(krate, "ori");
                syn::parse_quote!(#ori::core)
            }
            Err(_) => syn::parse_quote!(ori::core),
        },
    }
}

fn find_shell() -> syn::Path {
    match proc_macro_crate::crate_name("ori-shell") {
        Ok(krate) => found_crate(krate, "ori_shell"),
        Err(_) => match proc_macro_crate::crate_name("ori") {
            Ok(krate) => {
                let ori = found_crate(krate, "ori");
                syn::parse_quote!(#ori::shell)
            }
            Err(_) => syn::parse_quote!(ori::shell),
        },
    }
}

/// Get the inner type of an option.
///
/// Works for:
///  - `Option<T>`
///  - `option::Option<T>`
///  - `::std::option::Option<T>`
///  - `::core::option::Option<T>`
///  - `std::option::Option<T>`
///  - `core::option::Option<T>`
fn get_option_type(ty: &syn::Type) -> Option<&syn::Type> {
    const ALLOWED: &[&str] = &[
        "Option",
        "option::Option",
        "std::option::Option",
        "core::option::Option",
    ];

    let path = match ty {
        syn::Type::Path(path) => path,
        _ => return None,
    };

    let segments = &path.path.segments;

    let mut path_allowed = false;

    'outer: for allowed in ALLOWED {
        let allowed = allowed.split("::").collect::<Vec<_>>();

        if segments.len() != allowed.len() {
            continue;
        }

        for (segment, allowed) in segments.iter().zip(&allowed) {
            if segment.ident != allowed {
                continue 'outer;
            }
        }

        path_allowed = true;
    }

    if !path_allowed {
        return None;
    }

    let args = &segments.last()?.arguments;

    if let syn::PathArguments::AngleBracketed(args) = args {
        let args = &args.args;

        if args.len() != 1 {
            return None;
        }

        if let syn::GenericArgument::Type(ty) = args.first()? {
            return Some(ty);
        }
    }

    None
}

#[doc(hidden)]
#[proc_macro]
pub fn lowercase_ident(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let ident = syn::parse2::<syn::Ident>(input).unwrap();
    let ident = ident.to_string().to_lowercase();
    quote::quote!(#ident).into()
}

/// Load a font from a file or directory.
///
/// The path is relative to the `Cargo.toml` file.
#[manyhow::manyhow]
#[proc_macro]
pub fn include_font(input: proc_macro::TokenStream) -> manyhow::Result<proc_macro::TokenStream> {
    font::include_font(input)
}

/// Derive the `Rebuild` trait.
#[manyhow::manyhow]
#[proc_macro_derive(Rebuild, attributes(style, rebuild))]
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

/// Only include the annotated item on web platforms.
#[proc_macro_attribute]
pub fn web(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let expanded = quote::quote! {
        #[cfg(target_arch = "wasm32")]
        #input
    };

    expanded.into()
}

/// Check if the target platform is desktop.
#[proc_macro]
pub fn is_desktop(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let expanded = quote::quote! {
        ::std::cfg!(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "linux",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "netbsd",
        ))
    };

    expanded.into()
}

/// Check if the target platform is mobile.
#[proc_macro]
pub fn is_mobile(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let expanded = quote::quote! {
        ::std::cfg!(any(
            target_os = "android",
            target_os = "ios",
        ))
    };

    expanded.into()
}

/// Check if the target platform is web.
#[proc_macro]
pub fn is_web(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let expanded = quote::quote! {
        ::std::cfg!(target_arch = "wasm32")
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
