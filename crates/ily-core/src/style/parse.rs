use std::str::FromStr;

use ily_graphics::Color;
use pest::{error::Error, iterators::Pair, Parser};
use pest_derive::Parser;

use crate::{
    Attribute, AttributeValue, Style, StyleElement, StyleRule, StyleSelectors, Transition, Unit,
};

#[derive(Parser)]
#[grammar = "style/grammar.pest"]
pub struct StyleParser;

pub type StyleParseError = Error<Rule>;
pub type SelectorParseError = Error<Rule>;

fn parse_number(pair: Pair<'_, Rule>) -> f32 {
    pair.as_str().parse().unwrap()
}

fn parse_unit(pair: Pair<'_, Rule>) -> Unit {
    let mut pairs = pair.into_inner();

    let number_pair = pairs.next().unwrap();
    let number = parse_number(number_pair);

    match pairs.next().as_ref().map(Pair::as_rule) {
        Some(Rule::Px) | None => Unit::Px(number),
        Some(Rule::Pt) => Unit::Pt(number),
        Some(Rule::Pc) => Unit::Pc(number),
        _ => unreachable!(),
    }
}

fn parse_color(pair: Pair<'_, Rule>) -> Color {
    let pair = pair.into_inner().next().unwrap();

    match pair.as_rule() {
        Rule::HexColor => Color::hex(pair.as_str()),
        Rule::RgbColor => {
            let mut iter = pair.into_inner();

            let r = iter.next().unwrap().as_str().parse().unwrap();
            let g = iter.next().unwrap().as_str().parse().unwrap();
            let b = iter.next().unwrap().as_str().parse().unwrap();

            Color::rgb(r, g, b)
        }
        Rule::RgbaColor => {
            let mut iter = pair.into_inner();

            let r = iter.next().unwrap().as_str().parse().unwrap();
            let g = iter.next().unwrap().as_str().parse().unwrap();
            let b = iter.next().unwrap().as_str().parse().unwrap();
            let a = iter.next().unwrap().as_str().parse().unwrap();

            Color::rgba(r, g, b, a)
        }
        _ => unreachable!(),
    }
}

fn parse_transition(pair: Option<Pair<'_, Rule>>) -> Option<Transition> {
    Some(Transition::new(parse_number(pair?)))
}

fn parse_value(pair: Pair<'_, Rule>) -> (AttributeValue, Option<Transition>) {
    let mut pairs = pair.into_inner();

    let value = pairs.next().unwrap();
    let transition = parse_transition(pairs.next());
    let value = match value.as_rule() {
        Rule::String => AttributeValue::String(value.as_str().to_string()),
        Rule::Unit => AttributeValue::Length(parse_unit(value)),
        Rule::Color => AttributeValue::Color(parse_color(value)),
        _ => unreachable!(),
    };

    (value, transition)
}

fn parse_element(pair: Pair<'_, Rule>) -> StyleElement {
    let mut element = StyleElement::default();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::Identifier => {
                element.name = Some(pair.as_str().into());
            }
            Rule::Wildcard => {}
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

fn parse_selector(pair: Pair<'_, Rule>) -> StyleSelectors {
    let mut selector = StyleSelectors::default();

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
    let (value, transition) = parse_value(iter.next().unwrap());

    Attribute {
        name,
        value,
        transition,
    }
}

fn parse_style_rule(pair: Pair<'_, Rule>) -> StyleRule {
    let mut iter = pair.into_inner();

    let selector = parse_selector(iter.next().unwrap());
    let mut rule = StyleRule::new(selector);

    for pair in iter {
        match pair.as_rule() {
            Rule::Attribute => {
                rule.attributes.add(parse_attribute(pair));
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

impl FromStr for StyleSelectors {
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
