use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, parse_quote_spanned, spanned::Spanned};

use crate::{find_core, rebuild};

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
    let ident = &input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let style_name = syn::Ident::new(&format!("{}Style", ident), ident.span());
    let style_fields = style_fields(ident, &data.fields);
    let style_styled_fields = style_styled_fields(&data.fields);
    let style_rebuild_fields = style_rebuild_fields(&data.fields);

    let style_doc = format!("The derived style for [`{}`].", ident);
    let style_styled_doc = format!("The style of [`{}`].", ident);
    let style_rebuild_doc = format!("Rebuild the style of [`{}`].", ident);

    let expanded = quote! {
        #[doc = #style_doc]
        #[allow(unused)]
        #vis struct #style_name {
            #(#style_fields,)*
        }

        impl #style_name {
            #[doc = #style_styled_doc]
            #[allow(unused)]
            #vis fn styled #impl_generics (
                styled: &#ident #ty_generics,
                styles: &#ori_core::style::Styles
            ) -> Self
            #where_clause
            {
                Self {
                    #(#style_styled_fields,)*
                }
            }

            #[doc = #style_rebuild_doc]
            #[allow(unused)]
            #vis fn rebuild #impl_generics (
                &mut self,
                styled: &#ident #ty_generics,
                cx: &mut #ori_core::context::RebuildCx
            )
            #where_clause
            {
                let mut layout = false;
                let mut draw = false;
                let styles = cx.styles();

                #(#style_rebuild_fields)*

                if layout {
                    cx.layout();
                }

                if draw {
                    cx.draw();
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
    fields.iter().filter_map(move |field| {
        let ident = field.ident.as_ref().unwrap();
        let _ = get_styled(&field.ty)?;

        let styled = parse_quote!(styled);
        let styles = parse_quote!(styles);

        let value = style_get_field(field, &styled, &styles);

        Some(quote! {
            #ident: #value
        })
    })
}

fn style_rebuild_fields(fields: &syn::Fields) -> impl Iterator<Item = TokenStream> + '_ {
    fields.iter().filter_map(move |field| {
        let ident = field.ident.as_ref().unwrap();
        let _ = get_styled(&field.ty)?;

        let attrs = rebuild::FieldAttributes::new(&field.attrs).unwrap();

        if attrs.is_empty() {
            return None;
        }

        let styled = parse_quote!(styled);
        let styles = parse_quote!(styles);

        let value = style_get_field(field, &styled, &styles);

        let layout = match attrs.layout {
            true => Some(quote! {
                if self.#ident != value {
                    self.#ident = value;
                    layout = true;
                }
            }),
            false => None,
        };

        let draw = match attrs.draw {
            true => Some(quote! {
                if self.#ident != value {
                    self.#ident = value;
                    draw = true;
                }
            }),
            false => None,
        };

        Some(quote! {
            let value = #value;

            #layout
            #draw
        })
    })
}

fn style_get_field(field: &syn::Field, styled: &syn::Expr, styles: &syn::Expr) -> syn::Expr {
    let ori_core = find_core();
    let ident = field.ident.as_ref().unwrap();
    let mut default = None;

    for attr in &field.attrs {
        if attr.path().is_ident("styled") {
            attr.parse_args_with(|input: syn::parse::ParseStream| {
                if input.is_empty() {
                    return Ok(());
                }

                input.parse::<default>()?;

                if input.peek(syn::Token![=]) {
                    input.parse::<syn::Token![=]>()?;
                    default = Some(input.parse::<syn::Expr>()?);
                } else if input.peek(syn::Token![->]) {
                    input.parse::<syn::Token![->]>()?;
                    let style = input.parse::<syn::Expr>()?;

                    if input.peek(or) {
                        input.parse::<or>()?;
                        let or_value = input.parse::<syn::Expr>()?;
                        default = Some(parse_quote_spanned! { style.span() =>
                            #styles.get_or_else(|| #or_value, &#style)
                        });
                    } else {
                        default = Some(parse_quote_spanned! { style.span() =>
                            #styles.get(&#style).expect(concat!("missing style for `", #style, "`"))
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
        Some(default) => parse_quote! {
            #ori_core::style::Styled::get_or_else(
                &#styled.#ident,
                #styles,
                || #default
            )
        },
        None => parse_quote! {
            #ori_core::style::Styled::get(
                &#styled.#ident,
                #styles
            ).expect(concat!("missing style for `", stringify!(#ident), "`"))
        },
    }
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
