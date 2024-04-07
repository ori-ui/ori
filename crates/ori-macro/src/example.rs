use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Item, Token,
};

enum Arg {
    Name(String),
    Width(u32),
    Height(u32),
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;
        input.parse::<Token![=]>()?;

        match name.to_string().as_str() {
            "name" => {
                let name: syn::LitStr = input.parse()?;
                Ok(Self::Name(name.value()))
            }
            "width" => {
                let width: syn::LitInt = input.parse()?;
                Ok(Self::Width(width.base10_parse()?))
            }
            "height" => {
                let height: syn::LitInt = input.parse()?;
                Ok(Self::Height(height.base10_parse()?))
            }
            _ => Err(syn::Error::new(
                name.span(),
                "expected `name`, `width`, or `height`",
            )),
        }
    }
}

struct Args {
    name: String,
    width: u32,
    height: u32,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut width = None;
        let mut height = None;

        let items = Punctuated::<Arg, Token![,]>::parse_terminated(input)?;

        for item in items {
            match item {
                Arg::Name(value) => {
                    if name.is_some() {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            "duplicate `name` argument",
                        ));
                    }
                    name = Some(value);
                }
                Arg::Width(value) => {
                    if width.is_some() {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            "duplicate `width` argument",
                        ));
                    }
                    width = Some(value);
                }
                Arg::Height(value) => {
                    if height.is_some() {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            "duplicate `height` argument",
                        ));
                    }
                    height = Some(value);
                }
            }
        }

        Ok(Self {
            name: name.ok_or_else(|| syn::Error::new(input.span(), "missing `name` argument"))?,
            width: width
                .ok_or_else(|| syn::Error::new(input.span(), "missing `width` argument"))?,
            height: height
                .ok_or_else(|| syn::Error::new(input.span(), "missing `height` argument"))?,
        })
    }
}

pub fn example(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> manyhow::Result<proc_macro::TokenStream> {
    let args = syn::parse::<Args>(args)?;
    let mut input = syn::parse::<Item>(input)?;

    let iframe = format!(
        "<iframe 
            src=\"https://changecaps.github.io/ori-examples/?example={}\"
            style=\"width: {}px; height: {}px; border: none; overflow: hidden; border-radius: 6px;\"
        ></iframe>",
        args.name, args.width, args.height
    );

    match input {
        Item::Fn(ref mut item) => {
            item.attrs.push(syn::parse_quote!(#[doc = #iframe]));
        }
        Item::Struct(ref mut item) => {
            item.attrs.push(syn::parse_quote!(#[doc = #iframe]));
        }
        _ => manyhow::bail!("expected a function or a struct"),
    }

    Ok(input.into_token_stream().into())
}
