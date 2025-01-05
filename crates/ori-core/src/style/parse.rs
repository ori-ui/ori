use std::{
    any::Any,
    error::Error,
    fmt::{self, Debug, Display},
    iter::Peekable,
    ops::Range,
    str::FromStr,
    sync::Arc,
};

use crate::canvas::Color;

use super::{Style, Styled, Styles};

impl FromStr for Styles {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse(tokenize(s)?)
    }
}

/// An error that occurred while parsing a style.
pub struct ParseError {
    message: String,
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParseError: {}", self.message)
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error: {}", self.message)
    }
}

impl Error for ParseError {}

#[derive(Debug, PartialEq)]
enum TokenKind<'a> {
    Ident(&'a str),
    String(&'a str),
    Number(f32),
    Color(Color),
    True,
    False,
    Colon,
    Dot,
    Comma,
    OpenBrace,
    OpenBracket,
    CloseBrace,
    CloseBracket,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Token<'a> {
    kind: TokenKind<'a>,
    span: Range<usize>,
}

fn tokenize(s: &str) -> Result<impl ExactSizeIterator<Item = Token>, ParseError> {
    // i want a generator here, it's absense is making me sad

    let mut chars = s
        .chars()
        .map({
            let mut idx = 0;
            move |c| {
                idx += c.len_utf8();
                (c, idx - c.len_utf8())
            }
        })
        .peekable();

    let mut tokens = Vec::new();

    while let Some((c, lo)) = chars.next() {
        if c.is_whitespace() {
            continue;
        }

        if is_ident_start(c) {
            let mut hi = lo + c.len_utf8();

            while let Some(&(c, idx)) = chars.peek() {
                if !is_ident_continue(c) {
                    break;
                }

                hi = idx + c.len_utf8();
                chars.next();
            }

            let kind = match &s[lo..hi] {
                "true" => TokenKind::True,
                "false" => TokenKind::False,
                ident => TokenKind::Ident(ident),
            };

            tokens.push(Token { kind, span: lo..hi });

            continue;
        }

        if c.is_ascii_digit() {
            let mut hi = lo + c.len_utf8();

            while let Some(&(c, idx)) = chars.peek() {
                if s[lo..hi + c.len_utf8()].parse::<f32>().is_err() {
                    break;
                }

                hi = idx + c.len_utf8();
                chars.next();
            }

            let number = s[lo..hi].parse().unwrap_or(0.0);

            tokens.push(Token {
                kind: TokenKind::Number(number),
                span: lo..hi,
            });

            continue;
        }

        if c == '#' {
            let mut hi = lo + c.len_utf8();

            for (c, idx) in chars.by_ref() {
                if !c.is_ascii_hexdigit() {
                    break;
                }

                hi = idx + c.len_utf8();
            }

            let color = Color::hex(&s[lo + 1..hi]);

            tokens.push(Token {
                kind: TokenKind::Color(color),
                span: lo..hi,
            });

            continue;
        }

        if c == '"' {
            let mut hi = lo + c.len_utf8();

            for (c, idx) in chars.by_ref() {
                if c == '"' {
                    hi = idx + c.len_utf8();
                    break;
                }
            }

            tokens.push(Token {
                kind: TokenKind::String(&s[lo + 1..hi - 1]),
                span: lo..hi,
            });

            continue;
        }

        if let Some(kind) = get_symbol(c) {
            tokens.push(Token {
                kind,
                span: lo..lo + c.len_utf8(),
            });

            continue;
        }

        return Err(ParseError {
            message: format!("unexpected character: {:?}", c),
        });
    }

    Ok(tokens.into_iter())
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic()
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_'
}

fn get_symbol(c: char) -> Option<TokenKind<'static>> {
    match c {
        ':' => Some(TokenKind::Colon),
        '.' => Some(TokenKind::Dot),
        ',' => Some(TokenKind::Comma),
        '{' => Some(TokenKind::OpenBrace),
        '}' => Some(TokenKind::CloseBrace),
        '[' => Some(TokenKind::OpenBracket),
        ']' => Some(TokenKind::CloseBracket),
        _ => None,
    }
}

fn parse<'a, I>(tokens: I) -> Result<Styles, ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    let mut tokens = tokens.peekable();
    let mut styles = Styles::default();

    while tokens.peek().is_some() {
        parse_item(&mut tokens, "", &mut styles)?;
    }

    Ok(styles)
}

fn parse_item<'a, I>(
    tokens: &mut Peekable<I>,
    key: &str,
    styles: &mut Styles,
) -> Result<(), ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    let name = expect_ident(next(tokens)?)?;
    let token = next(tokens)?;

    let key = if key.is_empty() {
        name.to_string()
    } else {
        format!("{}.{}", key, name)
    };

    match token.kind {
        TokenKind::Colon => {
            let value = parse_value(tokens)?;
            styles.insert_entry(&key, value);

            if !is(tokens, TokenKind::Comma) {
                return Err(ParseError {
                    message: "expected ','".to_string(),
                });
            }

            next(tokens)?;

            Ok(())
        }

        TokenKind::OpenBrace => {
            while !is(tokens, TokenKind::CloseBrace) {
                parse_item(tokens, &key, styles)?;
            }

            next(tokens)?;

            Ok(())
        }

        _ => Err(ParseError {
            message: format!("expected ':' or '{{', found {:?}", token.kind),
        }),
    }
}

fn parse_value<'a, I>(
    tokens: &mut Peekable<I>,
) -> Result<Styled<Arc<dyn Any + Send + Sync>>, ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    let token = next(tokens)?;

    match token.kind {
        TokenKind::String(s) => Ok(Styled::Value(Arc::new(s.to_string()))),
        TokenKind::Number(n) => Ok(Styled::Value(Arc::new(n))),
        TokenKind::True => Ok(Styled::Value(Arc::new(true))),
        TokenKind::False => Ok(Styled::Value(Arc::new(false))),
        TokenKind::Color(color) => Ok(Styled::Value(Arc::new(color))),

        TokenKind::Ident(ident) => {
            let mut key = String::from(ident);

            while is(tokens, TokenKind::Dot) {
                next(tokens)?;
                let ident = expect_ident(next(tokens)?)?;
                key.push('.');
                key.push_str(ident);
            }

            Ok(Styled::Style(Style::from_string(key)))
        }

        TokenKind::OpenBracket => {
            let mut v = Vec::new();

            while !is(tokens, TokenKind::CloseBracket) {
                let token = next(tokens)?;

                match token.kind {
                    TokenKind::Number(n) => v.push(n),
                    _ => {
                        return Err(ParseError {
                            message: format!("expected number, found {:?}", token.kind),
                        });
                    }
                }

                if is(tokens, TokenKind::Comma) {
                    next(tokens)?;
                }
            }

            next(tokens)?;

            match v.len() {
                2 => Ok(Styled::Value(Arc::new([v[0], v[1]]))),
                4 => Ok(Styled::Value(Arc::new([v[0], v[1], v[2], v[3]]))),
                _ => Err(ParseError {
                    message: format!("expected 2 or 4 numbers, found {}", v.len()),
                }),
            }
        }

        _ => Err(ParseError {
            message: format!("expected string or number, found {:?}", token.kind),
        }),
    }
}

fn is<'a, I>(tokens: &mut Peekable<I>, kind: TokenKind) -> bool
where
    I: Iterator<Item = Token<'a>>,
{
    tokens.peek().map_or(false, |token| token.kind == kind)
}

fn next<'a, I>(tokens: &mut I) -> Result<Token<'a>, ParseError>
where
    I: Iterator<Item = Token<'a>>,
{
    tokens.next().ok_or(ParseError {
        message: "unexpected end of input".to_string(),
    })
}

fn expect_ident(token: Token<'_>) -> Result<&str, ParseError> {
    match token.kind {
        TokenKind::Ident(ident) => Ok(ident),
        _ => Err(ParseError {
            message: format!("expected identifier, found {:?}", token.kind),
        }),
    }
}
