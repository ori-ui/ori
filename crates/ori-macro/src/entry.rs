#[allow(unused_imports)]
use quote::{quote, ToTokens};

pub fn main(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> manyhow::Result<proc_macro::TokenStream> {
    let input = syn::parse::<syn::ItemFn>(input)?;

    let name = &input.sig.ident;
    let body = &input.block;
    let winit = crate::find_winit();

    let expanded = quote! {
        #[no_mangle]
        #[cfg(target_os = "android")]
        fn android_main(app: #winit::__private::AndroidApp) {
            #winit::__private::set_android_app(app);

            let body = || #body;
            body();
        }

        #input

        // this stops the compiler warning us that `main` is unused
        // when we're compiling a library target
        const _: fn() = #name;
    };

    Ok(expanded.into())
}
