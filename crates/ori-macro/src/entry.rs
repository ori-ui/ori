#[allow(unused_imports)]
use quote::{quote, ToTokens};

use crate::find_shell;

pub fn main(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> manyhow::Result<proc_macro::TokenStream> {
    let input = syn::parse::<syn::ItemFn>(input)?;

    let output = match input.sig.output {
        syn::ReturnType::Default => quote! { () },
        syn::ReturnType::Type(_, ref ty) => quote! { #ty },
    };

    let name = &input.sig.ident;
    let shell = find_shell();

    let expanded = quote! {
        #[no_mangle]
        #[cfg(target_os = "android")]
        fn android_main(android_app: #shell::platform::android::AndroidApp) -> #output {
            #shell::platform::android::ANDROID_APP.set(android_app).expect("Android app not set");

            self::#name()
        }

        #input
    };

    Ok(expanded.into())
}
