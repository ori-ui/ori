use std::str::FromStr;

use ily_graphics::Color;
use pest::{error::Error, iterators::Pair, Parser};
use pest_derive::Parser;

use crate::{
    Attribute, AttributeValue, Length, Pt, Px, Style, StyleElement, StyleRule, StyleSelector,
};

#[derive(Parser)]
#[grammar = "style/grammar.pest"]
pub struct StyleParser;

pub type StyleParseError = Error<Rule>;
pub type SelectorParseError = Error<Rule>;

fn parse_length(pair: Pair<'_, Rule>) -> Length {
    let number_pair = pair.clone().into_inner().next().unwrap();
    let number = number_pair.as_str().parse().unwrap();

    match pair.as_rule() {
        Rule::Px => Length::Px(Px(number)),
        Rule::Pt => Length::Pt(Pt(number)),
        _ => unreachable!(),
    }
}

fn parse_color(pair: Pair<'_, Rule>) -> Color {
    let pair = pair.into_inner().next().unwrap();

    match pair.as_rule() {
        Rule::HexColor => Color::hex(pair.as_str()),
        _ => unreachable!(),
    }
}

fn parse_value(pair: Pair<'_, Rule>) -> AttributeValue {
    match pair.as_rule() {
        Rule::String => AttributeValue::String(pair.as_str().to_string()),
        Rule::Px | Rule::Pt => AttributeValue::Length(parse_length(pair)),
        Rule::Color => AttributeValue::Color(parse_color(pair)),
        _ => unreachable!(),
    }
}

fn parse_element(pair: Pair<'_, Rule>) -> StyleElement {
    let mut iter = pair.into_inner();

    let mut element = StyleElement {
        name: iter.next().unwrap().as_str().into(),
        ..Default::default()
    };

    for pair in iter {
        match pair.as_rule() {
            Rule::State => {
                element.states.push(parse_class(pair));
            }
            _ => unreachable!(),
        }
    }

    element
}

fn parse_class(pair: Pair<'_, Rule>) -> String {
    pair.into_inner().as_str().to_string()
}

fn parse_selector(pair: Pair<'_, Rule>) -> StyleSelector {
    let mut selector = StyleSelector::default();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::Element => {
                selector.elements.push(parse_element(pair));
            }
            Rule::Class => {
                selector.classes.push(parse_class(pair));
            }
            Rule::Wildcard => {}
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
            Rule::Attribute => {
                rule.attributes.push(parse_attribute(pair));
            }
            _ => unreachable!(),
        }
    }

    rule
}

fn parse_style(input: &str) -> Result<Style, Error<Rule>> {
    let pairs = StyleParser::parse(Rule::Style, input)?.next().unwrap();
    let mut style = Style::new();

    for pair in pairs.into_inner() {
        match pair.as_rule() {
            Rule::StyleRule => {
                style.add_rule(parse_style_rule(pair));
            }
            Rule::EOI => break,
            _ => unreachable!(),
        }
    }

    Ok(style)
}

impl FromStr for Style {
    type Err = StyleParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        parse_style(input)
    }
}

impl FromStr for StyleSelector {
    type Err = SelectorParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let pair = StyleParser::parse(Rule::Selector, input)?.next().unwrap();
        Ok(parse_selector(pair))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        parse_style(include_str!("test.css")).unwrap();
    }
}
