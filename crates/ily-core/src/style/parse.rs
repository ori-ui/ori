use std::str::FromStr;

use ily_graphics::Color;
use pest::{error::Error, iterators::Pair, Parser};
use pest_derive::Parser;

use crate::{Attribute, AttributeValue, Length, Pt, Px, Selector, Style, StyleRule};

#[derive(Parser)]
#[grammar = "style/grammar.pest"]
pub struct StyleParser;

fn parse_length(pair: Pair<'_, Rule>) -> Length {
    let number = pair.clone().into_inner().as_str().parse().unwrap();

    match pair.as_rule() {
        Rule::px => Length::Px(Px(number)),
        Rule::pt => Length::Pt(Pt(number)),
        _ => unreachable!(),
    }
}

fn parse_color(pair: Pair<'_, Rule>) -> Color {
    match pair.as_rule() {
        Rule::hex_color => Color::hex(pair.as_str()),
        _ => unreachable!(),
    }
}

fn parse_value(pair: Pair<'_, Rule>) -> AttributeValue {
    match pair.as_rule() {
        Rule::string => AttributeValue::String(pair.as_str().to_string()),
        Rule::px | Rule::pt => AttributeValue::Length(parse_length(pair)),
        Rule::color => AttributeValue::Color(parse_color(pair)),
        _ => unreachable!("unhandled value: {:#?}", pair),
    }
}

fn parse_class(pair: Pair<'_, Rule>) -> String {
    pair.into_inner().as_str().to_string()
}

fn parse_selector(pair: Pair<'_, Rule>) -> Selector {
    let mut selector = Selector::default();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::element => {
                selector.element = Some(pair.as_str().to_string());
            }
            Rule::class => {
                selector.classes.push(parse_class(pair));
            }
            _ => unreachable!(),
        }
    }

    selector
}

fn parse_attribute(pair: Pair<'_, Rule>) -> Attribute {
    let mut iter = pair.into_inner();

    let name = iter.next().unwrap().as_str().to_string();
    let value = parse_value(iter.next().unwrap());

    Attribute { name, value }
}

fn parse_style_rule(pair: Pair<'_, Rule>) -> StyleRule {
    let mut iter = pair.into_inner();

    let selector = parse_selector(iter.next().unwrap());
    let mut rule = StyleRule::new(selector);

    for pair in iter {
        match pair.as_rule() {
            Rule::attribute => {
                rule.attributes.push(parse_attribute(pair));
            }
            _ => unreachable!(),
        }
    }

    rule
}

fn parse_stylesheet(input: &str) -> Result<Style, Error<Rule>> {
    let pairs = StyleParser::parse(Rule::stylesheet, input)?.next().unwrap();
    let mut style = Style::new();

    for pair in pairs.into_inner() {
        match pair.as_rule() {
            Rule::style_rule => {
                style.add_rule(parse_style_rule(pair));
            }
            Rule::EOI => break,
            _ => unreachable!(),
        }
    }

    Ok(style)
}

impl FromStr for Style {
    type Err = Error<Rule>;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        parse_stylesheet(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        parse_stylesheet(include_str!("test.css")).unwrap();
    }
}
