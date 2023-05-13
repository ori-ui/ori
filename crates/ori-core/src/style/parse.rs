use std::str::FromStr;

use ori_graphics::Color;
use pest::{error::Error, iterators::Pair, Parser};
use pest_derive::Parser;

use crate::{
    StyleAttribute, StyleAttributeKey, StyleAttributeValue, StyleClasses, StyleElement, StyleRule,
    StyleSelector, StyleSelectors, StyleStates, StyleTransition, Stylesheet, Unit,
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
        Some(Rule::Vw) => Unit::Vw(number),
        Some(Rule::Vh) => Unit::Vh(number),
        Some(Rule::Em) => Unit::Em(number),
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

fn parse_transition(pair: Option<Pair<'_, Rule>>) -> Option<StyleTransition> {
    Some(StyleTransition::new(parse_number(pair?)))
}

fn parse_value(pair: Pair<'_, Rule>) -> (StyleAttributeValue, Option<StyleTransition>) {
    let mut pairs = pair.into_inner();

    let value = pairs.next().unwrap();
    let transition = parse_transition(pairs.next());
    let value = match value.as_rule() {
        Rule::String => {
            let value = &value.as_str()[1..value.as_str().len() - 1];
            StyleAttributeValue::String(value.to_string())
        }
        Rule::Enum => StyleAttributeValue::Enum(value.as_str().to_string()),
        Rule::Unit => StyleAttributeValue::Unit(parse_unit(value)),
        Rule::Color => StyleAttributeValue::Color(parse_color(value)),
        _ => unreachable!(),
    };

    (value, transition)
}

fn parse_class(pair: Pair<'_, Rule>) -> String {
    pair.into_inner().as_str().to_string()
}

fn parse_state(pair: Pair<'_, Rule>) -> String {
    pair.into_inner().as_str().to_string()
}

fn parse_selector(pair: Pair<'_, Rule>) -> StyleSelector {
    let mut pairs = pair.into_inner();

    let mut element_pairs = pairs.next().unwrap().into_inner();
    let element_pair = element_pairs.next().unwrap();
    let element = match element_pair.as_rule() {
        Rule::Identifier => Some(StyleElement::new(element_pair.as_str())),
        Rule::Wildcard => None,
        _ => unreachable!(),
    };

    let mut selector = StyleSelector {
        element,
        classes: StyleClasses::new(),
        states: StyleStates::new(),
    };

    for pair in element_pairs {
        match pair.as_rule() {
            Rule::State => {
                selector.states.push(parse_state(pair));
            }
            _ => unreachable!(),
        }
    }

    for pair in pairs {
        match pair.as_rule() {
            Rule::Class => {
                selector.classes.push(parse_class(pair));
            }
            _ => unreachable!(),
        }
    }

    selector
}

fn parse_selectors(pair: Pair<'_, Rule>) -> StyleSelectors {
    let mut selectors = StyleSelectors::new();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::Selector => {
                selectors.push(parse_selector(pair));
            }
            _ => unreachable!(),
        }
    }

    selectors
}

fn parse_attribute(pair: Pair<'_, Rule>) -> StyleAttribute {
    let mut iter = pair.into_inner();

    let key = iter.next().unwrap().as_str();
    let (value, transition) = parse_value(iter.next().unwrap());

    let key = StyleAttributeKey::new(key);
    StyleAttribute::new(key, value, transition)
}

fn parse_style_rule(pair: Pair<'_, Rule>) -> StyleRule {
    let mut iter = pair.into_inner();

    let selector = parse_selectors(iter.next().unwrap());
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

fn parse_style(input: &str) -> Result<Stylesheet, Error<Rule>> {
    let pairs = StyleParser::parse(Rule::Style, input)?.next().unwrap();
    let mut style = Stylesheet::new();

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

impl FromStr for Stylesheet {
    type Err = StyleParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        parse_style(input)
    }
}

impl FromStr for StyleSelectors {
    type Err = SelectorParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let pair = StyleParser::parse(Rule::Selector, input)?.next().unwrap();
        Ok(parse_selectors(pair))
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
