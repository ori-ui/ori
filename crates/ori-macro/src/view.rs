use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, ResultExt};
use quote::{quote, quote_spanned};
use syn::{
    parse::{discouraged::Speculative, Parse, ParseStream, Parser},
    parse_quote, parse_quote_spanned,
    spanned::Spanned,
    Expr, ExprPath, Path, Stmt, Token,
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

/// Transforms a block into an iterator.
///
/// This introduces the `for` keyword to the view macro.
///
/// # Example
/// ```ignore
/// let items = vec![1, 2, 3];
///
/// view! {
///     <Div>
///         { for item }
///     </Div>
/// }
/// ```
fn transform_block(parser: ParseStream) -> Result<Option<TokenStream>, syn::Error> {
    let ori_core = find_crate("core");

    let content = parser.parse::<Content>()?;

    let tokens = match content {
        Content::For(expr) => quote!(::std::iter::IntoIterator::into_iter(#expr)),
        Content::Expr(expr) => quote!(::std::iter::once(#ori_core::Node::new(#expr))),
    };

    Ok(Some(tokens))
}

pub fn view(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let (context, rest) = parse_context(input.into());

    let config = syn_rsx::ParserConfig::new().transform_block(transform_block);
    let nodes = syn_rsx::parse2_with_config(rest, config).unwrap_or_abort();

    let expanded = if nodes.len() == 1 {
        create_element(&context, nodes.iter())
    } else {
        create_fragment(&context, nodes.iter())
    };

    expanded.into()
}

// parses
// $expr , $rest | $rest
fn parse_context(input: TokenStream) -> (Expr, TokenStream) {
    let parser = |parser: ParseStream| {
        if parser.is_empty() {
            return Ok((parse_quote!(cx), TokenStream::new()));
        }

        let fork = parser.fork();
        let Ok(expr) = fork.parse::<Expr>() else {
            return Ok((parse_quote!(cx), parser.parse()?));
        };

        if fork.peek(Token![,]) {
            fork.parse::<Token![,]>()?;
        } else {
            return Ok((parse_quote!(cx), parser.parse()?));
        }

        parser.advance_to(&fork);
        Ok((expr, parser.parse()?))
    };

    Parser::parse2(parser, input).unwrap_or_abort()
}

/// Creates an element node.
fn create_element<'a>(context: &Expr, nodes: impl Iterator<Item = &'a Node>) -> TokenStream {
    let nodes = nodes.map(|node| view_node(context, node));

    quote! {
        #(#nodes)*
    }
}

/// Pushes a dynamic node to the `__nodes` vector.
fn push_dynamic(context: &Expr, node: Expr) -> TokenStream {
    let ori_core = find_crate("core");

    quote! {
        let __dynamic = #context.owned_memo_scoped(move |#context| {
            let nodes = ::std::iter::Iterator::collect::<::std::vec::Vec<_>>(#node);
            #ori_core::Node::fragment(nodes)
        });

        __nodes.push(#ori_core::Node::dynamic(__dynamic));
    }
}

/// Pushes a static node to the `__nodes` vector.
pub fn push_static(node: Expr) -> TokenStream {
    quote! {
        __nodes.push(#node);
    }
}

/// Creates a fragment node.
fn create_fragment<'a>(context: &Expr, nodes: impl Iterator<Item = &'a Node>) -> TokenStream {
    let ori_core = find_crate("core");

    let elements = nodes.map(|node| {
        let is_dynamic = matches!(node, Node::Block(_));
        let node = view_node(context, node);

        if is_dynamic {
            push_dynamic(context, node)
        } else {
            push_static(node)
        }
    });

    quote! {{
        let mut __nodes = ::std::vec::Vec::new();
        #(#elements)*
        #ori_core::Node::fragment(__nodes)
    }}
}

/// Adds a child to an `__element` dynamically.
fn dynamic_child(context: &Expr, name: &Path, child: &Expr) -> TokenStream {
    let ori_core = find_crate("core");
    let ori_style = find_crate("style");

    let set_child = quote! {
        let _ = __element.downcast::<#ori_style::Styled<#name>, ()>(|__view| {
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
    };

    quote_spanned! {child.span() =>
        #context.effect_scoped({
            let __element = __element.clone();
            let mut __child_index = None;
            move |#context| { #set_child }
        });
    }
}

/// Adds a child to an `__element` statically.
fn static_child(name: &Path, child: &Expr) -> TokenStream {
    let ori_core = find_crate("core");
    let ori_style = find_crate("style");

    quote_spanned! {child.span() =>
        let _ = __element.downcast::<#ori_style::Styled<#name>, ()>(|__view| {
            <#name as #ori_core::Parent>::add_children(__view, ::std::iter::once(#child));
        });
    }
}

/// Adds children to an `__element`.
fn children<'a>(
    context: &'a Expr,
    name: Path,
    children: impl Iterator<Item = &'a Node> + 'a,
) -> impl Iterator<Item = TokenStream> + 'a {
    children.map(move |node| {
        let child = view_node(context, node);
        let is_dynamic = node_is_dynamic(node);

        if is_dynamic {
            dynamic_child(context, &name, &child)
        } else {
            static_child(&name, &child)
        }
    })
}

/// Returns an expression that evaluates to a `Node`.
fn view_node(context: &Expr, node: &Node) -> Expr {
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

            parse_quote_spanned! {element.name.span() => {
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
            parse_quote_spanned!(expr.span() =>
                #[allow(unused_braces)]
                #expr
            )
        }
        Node::Comment(comment) => {
            let comment = comment.value.as_ref();

            parse_quote_spanned! {comment.span() =>
                #ori_core::Node::new(#ori_core::Comment::new(#comment))
            }
        }
        Node::Text(text) => {
            let text = text.value.as_ref();

            parse_quote_spanned! {text.span() =>
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

/// Checks if the given expression is dynamic, which means that it can change reactively.
fn expr_is_dynamic(value: &Expr) -> bool {
    match value {
        Expr::Array(expr) => expr.elems.iter().any(expr_is_dynamic),
        Expr::Assign(_) | Expr::AssignOp(_) => false,
        Expr::Block(expr) => expr.block.stmts.iter().any(|stmt| match stmt {
            Stmt::Expr(expr) => expr_is_dynamic(expr),
            Stmt::Semi(expr, _) => expr_is_dynamic(expr),
            _ => true,
        }),
        Expr::Unary(expr) => expr_is_dynamic(&expr.expr),
        Expr::Binary(expr) => expr_is_dynamic(&expr.left) || expr_is_dynamic(&expr.right),
        Expr::Cast(expr) => expr_is_dynamic(&expr.expr),
        Expr::Closure(_) => false,
        Expr::Field(expr) => expr_is_dynamic(&expr.base),
        Expr::Group(expr) => expr_is_dynamic(&expr.expr),
        Expr::Index(expr) => expr_is_dynamic(&expr.expr) || expr_is_dynamic(&expr.index),
        Expr::Lit(_) => false,
        Expr::Paren(expr) => expr_is_dynamic(&expr.expr),
        Expr::Path(_) => false,
        Expr::Reference(expr) => expr_is_dynamic(&expr.expr),
        Expr::Repeat(expr) => expr_is_dynamic(&expr.expr),
        Expr::Return(expr) => match expr.expr.as_ref() {
            Some(expr) => expr_is_dynamic(expr),
            None => false,
        },
        Expr::Try(expr) => expr_is_dynamic(&expr.expr),
        Expr::Tuple(expr) => expr.elems.iter().any(expr_is_dynamic),
        Expr::Type(_) => false,
        _ => true,
    }
}

/// Checks if the given node is dynamic, which means that it can change reactively.
fn node_is_dynamic(node: &Node) -> bool {
    match node {
        Node::Block(block) => {
            let expr = block.value.as_ref();
            expr_is_dynamic(expr)
        }
        _ => false,
    }
}

/// Wraps the given block in an effect scope.
fn wrap_dynamic(context: &Expr, value: TokenStream) -> TokenStream {
    quote! {
        #context.effect_scoped({
            let __element = __element.clone();
            move |#context| { #value }
        });
    }
}

/// Sets the class of the given `__element`.
fn class(context: &Expr, name: &NodeName, value: &Expr) -> TokenStream {
    let ori_style = find_crate("style");

    let set_scope = quote_spanned! {value.span() =>
        let _ = __element.downcast::<#ori_style::Styled<#name>, ()>(|__view| {
            #ori_style::Styled::<#name>::set_class(__view, #value);
        });
    };

    if expr_is_dynamic(value) {
        wrap_dynamic(context, set_scope)
    } else {
        set_scope
    }
}

/// Sets a style attribute of the given `__element`.
fn style(context: &Expr, name: &NodeName, key: &str, value: &Expr) -> TokenStream {
    let ori_style = find_crate("style");

    let set_style = quote_spanned! {value.span() =>
        let _ = __element.downcast::<#ori_style::Styled<#name>, ()>(|__view| {
            #ori_style::Styled::<#name>::set_attr(__view, #key, #value);
        });
    };

    if expr_is_dynamic(value) {
        wrap_dynamic(context, set_style)
    } else {
        set_style
    }
}

/// Sets a property of the given `__element`.
fn property(context: &Expr, name: &NodeName, key: &ExprPath, value: &Expr) -> TokenStream {
    let ori_core = find_crate("core");
    let ori_style = find_crate("style");

    let key = quote_spanned! {key.path.span() =>
        #key
    };

    let set_property = quote_spanned! {value.span() =>
        let _ = __element.downcast::<#ori_style::Styled<#name>, ()>(|__view| {
            <#name as #ori_core::Properties>::setter(__view).#key(#value);
        });
    };

    if expr_is_dynamic(value) {
        wrap_dynamic(context, set_property)
    } else {
        set_property
    }
}

/// Sets an event handler of the given `__element`.
fn event(context: &Expr, name: &NodeName, key: &Ident, value: &Expr) -> TokenStream {
    let ori_core = find_crate("core");
    let ori_style = find_crate("style");

    let set_event = quote_spanned! {value.span() =>
        let _ = __element.downcast::<#ori_style::Styled<#name>, ()>(|__view| {
            <#name as #ori_core::Events>::setter(__view).#key(#context, #value);
        });
    };

    if expr_is_dynamic(value) {
        wrap_dynamic(context, set_event)
    } else {
        set_event
    }
}

/// Sets a binding of the given `__element`.
fn binding(context: &Expr, name: &NodeName, key: &Ident, value: &Expr) -> TokenStream {
    let ori_core = find_crate("core");
    let ori_style = find_crate("style");

    let set_binding = quote_spanned! {value.span() =>
        let _ = __element.downcast::<#ori_style::Styled<#name>, ()>(|__view| {
            <#name as #ori_core::Bindings>::setter(__view).#key(#context, #value);
        });
    };

    if expr_is_dynamic(value) {
        wrap_dynamic(context, set_binding)
    } else {
        set_binding
    }
}
