use proc_macro2::Span;
use quote::quote;

use crate::{find_core, find_reload};

fn get_view_data(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::ImplTrait(ref tr) = ty else {
        return None;
    };

    if tr.bounds.len() != 1 {
        return None;
    }

    let bound = &tr.bounds[0];

    match bound {
        syn::TypeParamBound::Trait(ref tb) => {
            let path = &tb.path;
            let segments = &path.segments;
            let segment = &segments[0];

            if segment.ident == "View" {
                let args = &segment.arguments;
                let args = match args {
                    syn::PathArguments::AngleBracketed(ref args) => &args.args,
                    _ => return None,
                };

                if args.len() != 1 {
                    return None;
                }

                let arg = &args[0];
                let ty = match arg {
                    syn::GenericArgument::Type(ty) => ty,
                    _ => return None,
                };

                return Some(ty.clone());
            }

            None
        }
        _ => None,
    }
}

pub fn reloadable(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> manyhow::Result<proc_macro::TokenStream> {
    let symbol = syn::parse::<syn::LitStr>(args)?;
    let input = syn::parse::<syn::ItemFn>(input)?;

    let symbol_name = syn::Ident::new(&symbol.value(), Span::call_site());
    let signature = &input.sig;
    let inputs = &signature.inputs;
    let block = &input.block;

    let data = match signature.output {
        syn::ReturnType::Type(_, ref ty) => get_view_data(ty),
        _ => {
            manyhow::bail!("reloadable functions must return `impl View`");
        }
    };

    if data.is_none() {
        manyhow::bail!("reloadable functions must return `impl View`");
    }

    let args = inputs
        .iter()
        .map(|arg| {
            let arg = match arg {
                syn::FnArg::Receiver(_) => {
                    manyhow::bail!("receiver argument is not supported");
                }
                syn::FnArg::Typed(arg) => arg,
            };

            let pat = &arg.pat;
            Ok::<_, manyhow::Error>(quote! { #pat })
        })
        .collect::<manyhow::Result<Vec<_>>>()?;

    let ori_core = find_core();
    let ori_reload = find_reload();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let target_dir = match cfg!(debug_assertions) {
        true => "debug",
        false => "release",
    };
    let name = std::env::var("CARGO_PKG_NAME").unwrap();

    #[cfg(target_os = "windows")]
    let path = format!("{}/target/{}/{}.dll", manifest_dir, target_dir, name);

    #[cfg(target_family = "unix")]
    let path = format!("{}/target/{}/lib{}.so", manifest_dir, target_dir, name);

    if cfg!(not(feature = "reload")) {
        let expanded = quote! {
            #input
        };

        return Ok(expanded.into());
    }

    let expanded = quote! {
        #signature {
            #[no_mangle]
            extern "C" fn #symbol_name(
                __styles: &mut #ori_core::style::Styles,
                #inputs
            ) -> #ori_core::view::BoxedView<#data> {
                __styles.as_context(|| {
                    ::std::boxed::Box::new({
                        #block
                    })
                })
            }

            static RELOADER: ::std::sync::Mutex<#ori_reload::Reloader> =
                ::std::sync::Mutex::new(#ori_reload::Reloader::new(#path));

            let mut __reloader = RELOADER.lock().unwrap();
            let mut __styles = #ori_core::style::Styles::snapshot();

            let __result = unsafe {
                let symbol = __reloader.load::<
                    extern "C" fn(
                        &mut #ori_core::style::Styles,
                        #inputs
                    ) -> #ori_core::view::BoxedView<#data>
                >(#symbol.as_bytes());

                match symbol {
                    Some(reloadable) => reloadable(&mut __styles, #(#args),*),
                    None => #symbol_name(&mut __styles, #(#args),*),
                }
            };

            #ori_core::style::Styles::context(|styles| *styles = __styles);

            #ori_reload::Watcher::new(__result, ::std::string::String::from(#path))
        }
    };

    Ok(expanded.into())
}
