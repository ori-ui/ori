use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, ResultExt};
use quote::{quote, quote_spanned};
use syn::{
    parse::{discouraged::Speculative, ParseStream, Parser},
    parse_quote,
    spanned::Spanned,
    Expr, ExprPath, Token,
};
use syn_rsx::{Node, NodeAttribute, NodeName};

pub fn view(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let (context, rest) = parse_context(input.into());
    let nodes = syn_rsx::parse2(rest).unwrap_or_abort();

    let nodes = nodes.iter().map(|node| view_node(&context, node));

    let expanded = if nodes.len() == 1 {
        quote! {
            #(#nodes)*
        }
    } else {
        quote! {
            ily::core::Div::zeroed()
                #( .child(#nodes) )*
        }
    };

    expanded.into()
}

// parses
// $expr , $rest | $rest
fn parse_context(input: TokenStream) -> (Expr, TokenStream) {
    let parser = |parser: ParseStream| {
        if parser.is_empty() {
            return Ok((parse_quote! { cx }, TokenStream::new()));
        }

        let fork = parser.fork();
        let Ok(expr) = fork.parse::<Expr>() else {
            return Ok((parse_quote! { cx }, parser.parse()?));
        };

        if fork.peek(Token![,]) {
            fork.parse::<Token![,]>()?;
        } else {
            return Ok((parse_quote! { cx }, parser.parse()?));
        }

        parser.advance_to(&fork);
        Ok((expr, parser.parse()?))
    };

    Parser::parse2(parser, input).unwrap_or_abort()
}

fn view_node(context: &Expr, node: &Node) -> TokenStream {
    match node {
        Node::Element(element) => {
            let name = &element.name;

            let mut attributes = Vec::new();
            let mut properties = Vec::new();

            for node in &element.attributes {
                let attr = get_attribute(&node);
                attribute(context, name, attr, &mut attributes, &mut properties);
            }

            let children = element.children.iter().map(|node| {
                let child = view_node(context, node);

                quote! {
                    <#name as ily::core::Parent>::add_child(&mut view, #child);
                }
            });

            quote! {
                ily::core::BoundedScope::dynamic(#context, move |#context| {
                    let mut view = <#name as ily::core::Styleable<_>>::styled(
                        <#name as ::std::default::Default>::default()
                    );

                    #(#properties)*
                    #(#attributes)*
                    #(#children)*

                    view
                })
            }
        }
        Node::Block(block) => {
            let expr = block.value.as_ref();
            quote_spanned!(expr.span() =>
                #[allow(unused_braces)]
                #expr
            )
        }
        Node::Comment(comment) => {
            let comment = comment.value.as_ref();

            quote! {
                ily::core::Comment::new(#comment)
            }
        }
        _ => unreachable!(),
    }
}

fn get_attribute(node: &Node) -> &NodeAttribute {
    let Node::Attribute(attribute) = node else {
        unreachable!()
    };

    attribute
}

fn attribute(
    context: &Expr,
    name: &NodeName,
    attr: &NodeAttribute,
    attributes: &mut Vec<TokenStream>,
    properties: &mut Vec<TokenStream>,
) {
    if let NodeName::Path(ref path) = attr.key {
        if let Some(ref value) = attr.value {
            properties.push(property(name, &path, &value));
        } else {
            properties.push(quote!(true));
        }

        return;
    }

    if let NodeName::Punctuated(ref punct) = attr.key {
        let (kind, key) = attribute_kind(attr);

        match kind.as_str() {
            "on" => {
                if let Some(ref value) = attr.value {
                    let key = Ident::new(&key, punct.span());
                    properties.push(event(context, name, &key, &value));
                } else {
                    abort!(punct, "expected event handler");
                }
            }
            "bind" => {
                if let Some(ref value) = attr.value {
                    let key = Ident::new(&key, punct.span());
                    properties.push(binding(context, name, &key, &value));
                } else {
                    abort!(punct, "expected binding");
                }
            }
            "style" => {
                if let Some(ref value) = attr.value {
                    attributes.push(style(context, name, &key, &value));
                } else {
                    abort!(punct, "expected attribute");
                }
            }
            _ => abort!(kind, "invalid attribute kind"),
        }

        return;
    }
}

fn attribute_kind(attribute: &NodeAttribute) -> (String, String) {
    let NodeName::Punctuated(ref punct) = attribute.key else {
        unreachable!()
    };

    let mut pairs = punct.pairs();

    let pair = pairs.next().unwrap();
    let kind = pair.value().clone();

    if pair.punct().unwrap().as_char() != ':' {
        abort!(punct, "expected ':'");
    }

    let mut key = String::new();
    for pair in pairs {
        let ident = pair.value().clone();

        key.push_str(&ident.to_string());

        if let Some(punct) = pair.punct() {
            if punct.as_char() != '-' {
                abort!(punct, "expected '-'");
            }

            key.push('-');
        }
    }

    if key.is_empty() {
        abort!(punct, "expected attribute name");
    }

    (kind.to_string(), key)
}

fn property(name: &NodeName, key: &ExprPath, value: &Expr) -> TokenStream {
    if key.path == parse_quote!(class) {
        return quote_spanned! {value.span() =>
            view = <ily::core::Styled<#name> as ily::core::Styleable<_>>::class(view, #value);
        };
    }

    let key = quote_spanned! {key.path.span() =>
        #key
    };

    quote_spanned! {value.span() =>
        <#name as ily::core::Properties>::setter(&mut view).#key(#value);
    }
}

fn event(context: &Expr, name: &NodeName, key: &Ident, value: &Expr) -> TokenStream {
    quote! {
        <#name as ily::core::Events>::setter(&mut view).#key(#context, #value);
    }
}

fn binding(context: &Expr, name: &NodeName, key: &Ident, value: &Expr) -> TokenStream {
    quote! {
        <#name as ily::core::Bindable>::setter(&mut view).#key(#context, #value);
    }
}

fn style(_context: &Expr, name: &NodeName, key: &str, value: &Expr) -> TokenStream {
    quote! {
        view = <ily::core::Styled<#name> as ily::core::Styleable<_>>::attr(view, #key, #value);
    }
}
