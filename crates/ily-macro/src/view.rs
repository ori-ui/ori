use proc_macro2::{TokenStream, Ident};
use proc_macro_error::{ResultExt, abort};
use quote::{quote, quote_spanned};
use syn::{
    parse::{ParseStream, Parser, discouraged::Speculative},
    parse_quote, Expr, ExprPath, Token, spanned::Spanned,
};
use syn_rsx::{Node, NodeName};


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

            let properties = element
                .attributes
                .iter()
                .map(|node| attribute(context, name, node));

            let children = element.children.iter().map(|node| {
                let child = view_node(context, node);

                quote! {
                    <#name as ily::core::Parent>::add_child(
                        &mut view,
                        ily::core::Child::new(#child)
                    );
                }
            });

            quote! {
                ily::core::BoundedScope::dynamic(#context, |#context| {
                    let mut view = <#name as ::std::default::Default>::default();

                    #(#properties)*
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

fn attribute(context: &Expr, name: &NodeName, attribute: &Node) -> TokenStream {
    let Node::Attribute(attribute) = attribute else {
        unreachable!()
    };

    match attribute.key {
        NodeName::Path(ref path) => {
            if let Some(ref value) = attribute.value {
                property(name, path, value)
            } else {
                unimplemented!()
            }
        }
        NodeName::Punctuated(ref punct) => {
            if punct.len() > 2 {
                abort!(punct, "invalid event name");
            }

            let pair = punct.pairs().next().unwrap();

            let kind = pair.value();

            if pair.punct().unwrap().as_char() != ':' {
                abort!(punct, "expected ':'");
            }

            let key = punct.last().unwrap(); 

            let Some(ref value) = attribute.value else {
                abort!(punct, "expected value");
            };
            
            if *kind == "on" {
                return event(context, name, key, value);
            }

            abort!(kind, "expected 'on'");
        }
        _ => unimplemented!(),
    }
}

fn property(name: &NodeName, key: &ExprPath, value: &Expr) -> TokenStream {
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
