use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, ResultExt};
use quote::{quote, quote_spanned};
use syn::{
    parse::{discouraged::Speculative, Parse, ParseStream, Parser},
    parse_quote,
    spanned::Spanned,
    Expr, ExprPath, Path, Token,
};
use syn_rsx::{Node, NodeAttribute, NodeName};

use crate::krate::find_crate;

enum Content {
    For(Expr),
    Expr(Expr),
}

impl Parse for Content {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fork = input.fork();
        if fork.parse::<Token![for]>().is_ok() {
            let expr = fork.parse::<Expr>()?;
            input.advance_to(&fork);
            Ok(Self::For(expr))
        } else {
            let expr = input.parse::<Expr>()?;
            Ok(Self::Expr(expr))
        }
    }
}

pub fn view(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let (context, rest) = parse_context(input.into());
    let ori_core = find_crate("core");

    let config = syn_rsx::ParserConfig::new().transform_block(move |parser| {
        let content = parser.parse::<Content>()?;

        let tokens = match content {
            Content::For(expr) => quote!(::std::iter::IntoIterator::into_iter(#expr)),
            Content::Expr(expr) => quote!(::std::iter::once(#ori_core::Node::new(#expr))),
        };

        Ok(Some(tokens))
    });

    let ori_core = find_crate("core");
    let nodes = syn_rsx::parse2_with_config(rest, config).unwrap_or_abort();

    let expanded = if nodes.len() == 1 {
        let nodes = nodes.iter().map(|node| view_node(&context, node));

        quote! {
            #(#nodes)*
        }
    } else {
        let elements = nodes.iter().map(|node| {
            let element = view_node(&context, node);
            let is_dynamic = matches!(node, Node::Block(_));

            if is_dynamic {
                quote! {
                    let __dynamic = #context.owned_memo_scoped(move |#context| {
                        let nodes = ::std::iter::Iterator::collect::<::std::vec::Vec<_>>(#element);
                        #ori_core::Node::fragment(nodes)
                    });

                    __nodes.push(#ori_core::Node::dynamic(__dynamic));
                }
            } else {
                quote! {
                    __nodes.push(#element);
                }
            }
        });

        quote! {{
            let mut __nodes = ::std::vec::Vec::new();

            #(#elements)*

            #ori_core::Node::fragment(__nodes)
        }}
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

fn children<'a>(
    context: &'a Expr,
    name: Path,
    children: impl Iterator<Item = &'a Node> + 'a,
) -> impl Iterator<Item = TokenStream> + 'a {
    let ori_core = find_crate("core");
    let ori_style = find_crate("style");

    children.map(move |node| {
        let child = view_node(context, node);
        let is_dynamic = matches!(node, Node::Block(_));

        if is_dynamic {
            quote! {
                #context.effect_scoped({
                    let __element = __element.clone();
                    let mut __child_index = None;
                    move |#context| {
                        let _ = __element.with_view::<#ori_style::Styled<#name>, ()>(|__view| {
                            if let Some(__child_index) = __child_index {
                                <#name as #ori_core::Parent>::set_children(
                                    __view,
                                    __child_index,
                                    #child,
                                );
                            } else {
                                __child_index = Some(<#name as #ori_core::Parent>::add_children(
                                    __view,
                                    #child,
                                ));
                            }
                        });
                    }
                });
            }
        } else {
            quote! {{
                let _ = __element.with_view::<#ori_style::Styled<#name>, ()>(|__view| {
                    <#name as #ori_core::Parent>::add_children(__view, ::std::iter::once(#child));
                });
            }}
        }
    })
}

fn view_node(context: &Expr, node: &Node) -> TokenStream {
    let ori_core = find_crate("core");
    let ori_style = find_crate("style");

    match node {
        Node::Element(element) => {
            let name = &element.name;

            let mut attributes = Vec::new();
            let mut properties = Vec::new();

            for node in &element.attributes {
                let attr = get_attribute(node);
                attribute(context, name, attr, &mut attributes, &mut properties);
            }

            let children = children(context, parse_quote!(#name), element.children.iter());

            quote! {{
                let mut __view = <#name as #ori_style::Stylable<_>>::styled(
                    <#name as ::std::default::Default>::default()
                );
                let __element = #ori_core::Element::new(__view);

                #(#children)*
                #(#properties)*
                #(#attributes)*

                #ori_core::Node::element(__element)
            }}
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
                #ori_core::Comment::new(#comment)
            }
        }
        Node::Text(text) => {
            let text = text.value.as_ref();

            quote_spanned! {text.span() =>
                #ori_core::Node::new(#text)
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
        let Some(ref value) = attr.value else { return };

        if path.path == parse_quote!(class) {
            attributes.push(class(context, name, value));
            return;
        }

        properties.push(property(context, name, path, value));
        return;
    }

    if let NodeName::Punctuated(ref punct) = attr.key {
        let (kind, key) = attribute_kind(attr);

        match kind.as_str() {
            "on" => {
                if let Some(ref value) = attr.value {
                    let key = Ident::new(&key, punct.span());
                    properties.push(event(context, name, &key, value));
                } else {
                    abort!(punct, "expected event handler");
                }
            }
            "bind" => {
                if let Some(ref value) = attr.value {
                    let key = Ident::new(&key, punct.span());
                    properties.push(binding(context, name, &key, value));
                } else {
                    abort!(punct, "expected binding");
                }
            }
            "style" => {
                if let Some(ref value) = attr.value {
                    attributes.push(style(context, name, &key, value));
                } else {
                    abort!(punct, "expected attribute");
                }
            }
            _ => abort!(kind, "invalid attribute kind"),
        }
    }
}

fn attribute_kind(attribute: &NodeAttribute) -> (String, String) {
    let NodeName::Punctuated(ref punct) = attribute.key else {
        unreachable!()
    };

    let mut pairs = punct.pairs();

    let pair = pairs.next().unwrap();
    let kind = pair.value();

    if pair.punct().unwrap().as_char() != ':' {
        abort!(punct, "expected ':'");
    }

    let mut key = String::new();
    for pair in pairs {
        let ident = pair.value();

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

fn is_dynamic(value: &Expr) -> bool {
    match value {
        Expr::Array(expr) => expr.elems.iter().any(is_dynamic),
        Expr::Assign(_) | Expr::AssignOp(_) => false,
        Expr::Unary(expr) => is_dynamic(&expr.expr),
        Expr::Binary(expr) => is_dynamic(&expr.left) || is_dynamic(&expr.right),
        Expr::Cast(expr) => is_dynamic(&expr.expr),
        Expr::Closure(_) => false,
        Expr::Field(expr) => is_dynamic(&expr.base),
        Expr::Group(expr) => is_dynamic(&expr.expr),
        Expr::Index(expr) => is_dynamic(&expr.expr) || is_dynamic(&expr.index),
        Expr::Lit(_) => false,
        Expr::Paren(expr) => is_dynamic(&expr.expr),
        Expr::Path(_) => false,
        Expr::Reference(expr) => is_dynamic(&expr.expr),
        Expr::Repeat(expr) => is_dynamic(&expr.expr),
        Expr::Try(expr) => is_dynamic(&expr.expr),
        Expr::Tuple(expr) => expr.elems.iter().any(is_dynamic),
        Expr::Type(_) => false,
        _ => true,
    }
}

fn wrap_effect(context: &Expr, value: TokenStream) -> TokenStream {
    quote! {
        #context.effect_scoped({
            let __element = __element.clone();
            move |#context| { #value }
        });
    }
}

fn class(context: &Expr, name: &NodeName, value: &Expr) -> TokenStream {
    let ori_style = find_crate("style");

    let tt = quote_spanned! {value.span() =>
        let _ = __element.with_view::<#ori_style::Styled<#name>, ()>(|__view| {
            #ori_style::Styled::<#name>::set_class(__view, #value);
        });
    };

    if is_dynamic(value) {
        wrap_effect(context, tt)
    } else {
        tt
    }
}

fn property(context: &Expr, name: &NodeName, key: &ExprPath, value: &Expr) -> TokenStream {
    let ori_core = find_crate("core");
    let ori_style = find_crate("style");

    let key = quote_spanned! {key.path.span() =>
        #key
    };

    let tt = quote_spanned! {value.span() =>
        let _ = __element.with_view::<#ori_style::Styled<#name>, ()>(|__view| {
            <#name as #ori_core::Properties>::setter(__view).#key(#value);
        });
    };

    if is_dynamic(value) {
        wrap_effect(context, tt)
    } else {
        tt
    }
}

fn event(context: &Expr, name: &NodeName, key: &Ident, value: &Expr) -> TokenStream {
    let ori_core = find_crate("core");
    let ori_style = find_crate("style");

    let tt = quote_spanned! {value.span() =>
        let _ = __element.with_view::<#ori_style::Styled<#name>, ()>(|__view| {
            <#name as #ori_core::Events>::setter(__view).#key(#context, #value);
        });
    };

    if is_dynamic(value) {
        wrap_effect(context, tt)
    } else {
        tt
    }
}

fn binding(context: &Expr, name: &NodeName, key: &Ident, value: &Expr) -> TokenStream {
    let ori_core = find_crate("core");
    let ori_style = find_crate("style");

    let tt = quote_spanned! {value.span() =>
        let _ = __element.with_view::<#ori_style::Styled<#name>, ()>(|__view| {
            <#name as #ori_core::Bindings>::setter(__view).#key(#context, #value);
        });
    };

    if is_dynamic(value) {
        wrap_effect(context, tt)
    } else {
        tt
    }
}

fn style(context: &Expr, name: &NodeName, key: &str, value: &Expr) -> TokenStream {
    let ori_style = find_crate("style");

    let tt = quote_spanned! {value.span() =>
        let _ = __element.with_view::<#ori_style::Styled<#name>, ()>(|__view| {
            #ori_style::Styled::<#name>::set_attr(__view, #key, #value);
        });
    };

    if is_dynamic(value) {
        wrap_effect(context, tt)
    } else {
        tt
    }
}
