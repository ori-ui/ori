use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::ParseStream, punctuated::Punctuated, DeriveInput};

syn::custom_keyword!(ignore);

enum FieldAttribute {
    Ignore,
}

impl syn::parse::Parse for FieldAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(ignore) {
            input.parse::<ignore>()?;
            Ok(Self::Ignore)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Default)]
struct FieldAttributes {
    ignore: bool,
}

impl FieldAttributes {
    fn new(attrs: &[syn::Attribute]) -> manyhow::Result<Self> {
        let mut this = Self::default();

        for attr in attrs {
            if attr.path().is_ident("build") {
                let updates = attr.parse_args_with(|input: ParseStream| {
                    Punctuated::<FieldAttribute, syn::Token![,]>::parse_terminated(input)
                })?;

                for update in updates {
                    match update {
                        FieldAttribute::Ignore => this.ignore = true,
                    }
                }
            }
        }

        Ok(this)
    }
}

pub fn derive_build(input: proc_macro::TokenStream) -> manyhow::Result<proc_macro::TokenStream> {
    let input = syn::parse::<DeriveInput>(input)?;

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let build = build_impl(&input)?;

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(#build)*
        }
    };

    Ok(expanded.into())
}

fn build_impl(input: &syn::DeriveInput) -> manyhow::Result<Vec<TokenStream>> {
    match input.data {
        syn::Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                let names = named_fields(fields);
                build_fields(names, fields.named.iter())
            }
            syn::Fields::Unnamed(_) => {
                manyhow::bail!("`Build` cannot be derived for tuple structs")
            }
            syn::Fields::Unit => Ok(Vec::new()),
        },
        syn::Data::Enum(_) => manyhow::bail!("`Build` cannot be derived for enums"),
        syn::Data::Union(_) => manyhow::bail!("`Build` cannot be derived for unions"),
    }
}

fn named_fields(fields: &syn::FieldsNamed) -> impl Iterator<Item = TokenStream> + '_ {
    fields.named.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        quote!(#ident)
    })
}

fn build_fields<'a>(
    names: impl Iterator<Item = TokenStream>,
    fields: impl Iterator<Item = &'a syn::Field>,
) -> manyhow::Result<Vec<TokenStream>> {
    let mut build = Vec::new();

    for (name, field) in names.zip(fields) {
        build.push(build_field(name, field)?);
    }

    Ok(build)
}

fn build_field(name: TokenStream, field: &syn::Field) -> manyhow::Result<TokenStream> {
    let attrs = FieldAttributes::new(&field.attrs)?;

    if attrs.ignore {
        return Ok(quote!());
    }

    let ty = &field.ty;
    let doc = format!("Set `self.{}`.", name);

    let mut field_doc = Vec::new();

    for attr in &field.attrs {
        if attr.path().is_ident("doc") {
            field_doc.push(attr);
        }
    }

    Ok(quote! {
        #[doc = #doc]
        #[doc = ""]
        #(#field_doc)*
        pub fn #name(mut self, #name: impl Into<#ty>) -> Self {
            self.#name = ::std::convert::Into::into(#name);
            self
        }
    })
}
