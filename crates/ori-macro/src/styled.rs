use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, parse_quote_spanned};

use crate::find_core;

syn::custom_keyword!(or);
syn::custom_keyword!(default);

pub fn derive_styled(input: proc_macro::TokenStream) -> manyhow::Result<proc_macro::TokenStream> {
    let input = syn::parse::<syn::DeriveInput>(input)?;

    let syn::Data::Struct(ref data) = input.data else {
        manyhow::bail!("`Styled` can only be derived for structs");
    };

    if !matches!(data.fields, syn::Fields::Named(_)) {
        manyhow::bail!("`Styled` can only be derived for structs with named fields");
    }

    let ori_core = find_core();

    let vis = &input.vis;
    let name = &input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let style_name = syn::Ident::new(&format!("{}Style", name), name.span());
    let style_fields = style_fields(&name, &data.fields);
    let style_styled_fields = style_styled_fields(&data.fields);

    let style_doc = format!("The derived style for [`{}`].", name);
    let style_styled_doc = format!("The style of [`{}`].", name);

    let expanded = quote! {
        #[doc = #style_doc]
        #vis struct #style_name {
            #(#style_fields,)*
        }

        impl #style_name {
            #[doc = #style_styled_doc]
            #vis fn styled #impl_generics (
                styled: &#name #ty_generics,
                styles: &#ori_core::style::Styles
            ) -> Self
            #where_clause
            {
                Self {
                    #(#style_styled_fields,)*
                }
            }
        }
    };

    Ok(expanded.into())
}

fn style_fields<'a>(
    name: &'a proc_macro2::Ident,
    fields: &'a syn::Fields,
) -> impl Iterator<Item = TokenStream> + 'a {
    fields.iter().filter_map(move |field| {
        let vis = &field.vis;
        let ident = field.ident.as_ref().unwrap();
        let ty = get_styled(&field.ty)?;

        let doc = format!("The style of [`{}::{}`].", name, ident);

        Some(quote! {
            #[doc = #doc]
            #vis #ident: #ty
        })
    })
}

fn style_styled_fields(fields: &syn::Fields) -> impl Iterator<Item = TokenStream> + '_ {
    let ori_core = find_core();

    fields.iter().filter_map(move |field| {
        let ident = field.ident.as_ref().unwrap();
        let _ = get_styled(&field.ty)?;

        let mut default = None;

        for attr in &field.attrs {
            if attr.path().is_ident("styled") {
                attr.parse_args_with(|input: syn::parse::ParseStream| {
                    input.parse::<default>()?;

                    if input.peek(syn::Token![=]) {
                        input.parse::<syn::Token![=]>()?;
                        default = Some(input.parse::<syn::Expr>()?);
                    } else if input.peek(syn::Token![->]) {
                        input.parse::<syn::Token![->]>()?;
                        let style = input.parse::<syn::LitStr>()?;

                        if input.peek(or) {
                            input.parse::<or>()?;
                            let or_value = input.parse::<syn::Expr>()?;
                            default = Some(parse_quote_spanned! { style.span() =>
                                styles.get_or_else(|| #or_value, #style)
                            });
                        } else {
                            default = Some(parse_quote_spanned! { style.span() =>
                                styles.get(#style).expect(concat!("missing style for `", #style, "`"))
                            });
                        }
                    } else {
                        default = Some(parse_quote!(::std::default::Default::default()));
                    }

                    Ok(())
                })
                .unwrap();
            }
        }

        match default {
            Some(default) => Some(quote! {
                #ident: #ori_core::style::Styled::get_or_else(
                    ::std::clone::Clone::clone(&styled.#ident),
                    styles,
                    || #default
                )
            }),
            None => Some(quote! {
                #ident: #ori_core::style::Styled::get(
                    ::std::clone::Clone::clone(&styled.#ident),
                    styles
                ).expect(concat!("missing style for `", stringify!(#ident), "`"))
            }),
        }
    })
}

fn get_styled(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(ty) = ty else {
        return None;
    };

    if ty.qself.is_some() {
        return None;
    }

    match ty.path.segments.len() {
        1 => {
            let segment = &ty.path.segments[0];

            if segment.ident != "Styled" {
                return None;
            }

            match &segment.arguments {
                syn::PathArguments::AngleBracketed(args) if args.args.len() == 1 => {
                    let arg = args.args.first().unwrap();

                    match arg {
                        syn::GenericArgument::Type(ty) => Some(ty.clone()),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}
