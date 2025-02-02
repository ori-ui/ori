use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::ParseStream, punctuated::Punctuated};

use crate::find_core;

syn::custom_keyword!(layout);
syn::custom_keyword!(draw);

enum FieldAttribute {
    Layout,
    Draw,
}

impl syn::parse::Parse for FieldAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(layout) {
            input.parse::<layout>()?;
            Ok(Self::Layout)
        } else if lookahead.peek(draw) {
            input.parse::<draw>()?;
            Ok(Self::Draw)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Default)]
pub struct FieldAttributes {
    pub styled: bool,
    pub layout: bool,
    pub draw: bool,
}

impl FieldAttributes {
    pub fn new(attrs: &[syn::Attribute]) -> manyhow::Result<Self> {
        let mut this = Self::default();

        for attr in attrs {
            if attr.path().is_ident("rebuild") {
                let updates = attr.parse_args_with(|input: ParseStream| {
                    Punctuated::<FieldAttribute, syn::Token![,]>::parse_terminated(input)
                })?;

                for update in updates {
                    match update {
                        FieldAttribute::Layout => this.layout = true,
                        FieldAttribute::Draw => this.draw = true,
                    }
                }
            }

            if attr.path().is_ident("style") {
                this.styled = true;
            }
        }

        Ok(this)
    }

    pub fn is_empty(&self) -> bool {
        !self.layout && !self.draw
    }

    fn updates(&self) -> TokenStream {
        let mut tokens = TokenStream::new();

        if self.layout {
            tokens.extend(quote!(cx.layout();));
        }

        if self.draw {
            tokens.extend(quote!(cx.draw();));
        }

        tokens
    }
}

pub fn derive_rebuild(input: proc_macro::TokenStream) -> manyhow::Result<proc_macro::TokenStream> {
    let input: syn::DeriveInput = syn::parse(input)?;

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let ori_core = find_core();

    let rebuild_impl = rebuild_impl(&input)?;

    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics #ori_core::rebuild::Rebuild for #name #ty_generics #where_clause {
            #[allow(unused)]
            fn rebuild(&self, cx: &mut #ori_core::context::RebuildCx, old: &Self) {
                #rebuild_impl
            }
        }
    };

    Ok(expanded.into())
}

fn rebuild_impl(input: &syn::DeriveInput) -> manyhow::Result<TokenStream> {
    match input.data {
        syn::Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                let names = named_fields(fields);
                rebuild_fields(names, fields.named.iter())
            }
            syn::Fields::Unnamed(ref fields) => {
                let names = unnamed_fields(fields);
                rebuild_fields(names, fields.unnamed.iter())
            }
            syn::Fields::Unit => Ok(quote!()),
        },
        syn::Data::Enum(_) => manyhow::bail!("enums are not supported"),
        syn::Data::Union(_) => manyhow::bail!("unions are not supported"),
    }
}

fn named_fields(fields: &syn::FieldsNamed) -> impl Iterator<Item = TokenStream> + '_ {
    fields.named.iter().map(|field| {
        let name = field.ident.as_ref().unwrap();
        quote!(#name)
    })
}

fn unnamed_fields(fields: &syn::FieldsUnnamed) -> impl Iterator<Item = TokenStream> + '_ {
    fields.unnamed.iter().enumerate().map(|(i, _)| {
        let i = syn::Index::from(i);
        quote!(#i)
    })
}

fn rebuild_fields<'a>(
    names: impl Iterator<Item = TokenStream>,
    fields: impl Iterator<Item = &'a syn::Field>,
) -> manyhow::Result<TokenStream> {
    let mut tokens = TokenStream::new();

    for (name, field) in names.zip(fields) {
        tokens.extend(rebuild_field(name, field)?);
    }

    Ok(tokens)
}

fn rebuild_field(name: TokenStream, field: &syn::Field) -> manyhow::Result<TokenStream> {
    let attributes = FieldAttributes::new(&field.attrs)?;

    if attributes.is_empty() || attributes.styled {
        return Ok(quote!());
    }

    let updates = attributes.updates();
    Ok(quote! {
        if self.#name != old.#name {
            #updates
        }
    })
}
