use std::str::FromStr;

use ori_graphics::Color;
use pest::{error::Error, iterators::Pair, Parser};
use pest_derive::Parser;

use crate::{
    Length, StyleAttribute, StyleAttributeKey, StyleAttributeValue, StyleClasses, StyleElement,
    StyleElementSelector, StyleSelector, StyleSelectorCombinator, StyleTags, StyleTransition,
    Stylesheet, StylesheetRule,
};

/// A parser for [`Stylesheet`]s.
#[derive(Parser)]
#[grammar = "parse/grammar.pest"]
pub struct StyleParser;

/// An error that occurred while parsing a [`Stylesheet`].
pub type StyleheetParseError = Box<Error<Rule>>;

fn parse_number(pair: Pair<'_, Rule>) -> f32 {
    pair.as_str().parse().unwrap()
}

fn parse_length(pair: Pair<'_, Rule>) -> Length {
    let mut pairs = pair.into_inner();

    let number_pair = pairs.next().unwrap();
    let number = parse_number(number_pair);

    match pairs.next().as_ref().map(Pair::as_rule) {
        Some(Rule::Px) | None => Length::Px(number),
        Some(Rule::Pt) => Length::Pt(number),
        Some(Rule::Pc) => Length::Pc(number),
        Some(Rule::Vw) => Length::Vw(number),
        Some(Rule::Vh) => Length::Vh(number),
        Some(Rule::Em) => Length::Em(number),
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
        Rule::Length => StyleAttributeValue::Length(parse_length(value)),
        Rule::Color => StyleAttributeValue::Color(parse_color(value)),
        _ => unreachable!(),
    };

    (value, transition)
}

fn parse_class(pair: Pair<'_, Rule>) -> String {
    pair.into_inner().as_str().to_string()
}

fn parse_tag(pair: Pair<'_, Rule>) -> String {
    pair.into_inner().as_str().to_string()
}

fn parse_element_selector(pair: Pair<'_, Rule>) -> StyleElementSelector {
    let mut pairs = pair.into_inner();

    let mut element_pairs = pairs.next().unwrap().into_inner();
    let element_pair = element_pairs.next().unwrap();
    let element = match element_pair.as_rule() {
        Rule::Identifier => Some(StyleElement::new(element_pair.as_str())),
        Rule::Wildcard => None,
        _ => unreachable!(),
    };

    let mut selector = StyleElementSelector {
        element,
        classes: StyleClasses::new(),
        tags: StyleTags::new(),
    };

    for pair in pairs {
        match pair.as_rule() {
            Rule::Class => {
                selector.classes.push(parse_class(pair));
            }
            Rule::Tag => {
                selector.tags.push(parse_tag(pair));
            }
            _ => unreachable!(),
        }
    }

    selector
}

fn parse_combinator(pair: Pair<'_, Rule>) -> StyleSelectorCombinator {
    let pair = pair.into_inner().next().unwrap();

    match pair.as_rule() {
        Rule::Descendant => StyleSelectorCombinator::Descendant,
        Rule::Child => StyleSelectorCombinator::Child,
        _ => unreachable!("{:?} is not a combinator", pair),
    }
}

fn parse_selector(pair: Pair<'_, Rule>) -> StyleSelector {
    let mut iter = pair.into_inner();

    let mut combinators = Vec::new();
    let mut element = parse_element_selector(iter.next().unwrap());

    loop {
        let Some(pair) = iter.next() else {
            break;
        };

        let combinator = parse_combinator(pair);
        let element_selector = parse_element_selector(iter.next().unwrap());
        combinators.push((element, combinator));
        element = element_selector;
    }

    StyleSelector {
        combinators: combinators.into(),
        element,
    }
}

fn parse_selectors(pair: Pair<'_, Rule>) -> Vec<StyleSelector> {
    let mut selectors = Vec::new();

    for pair in pair.into_inner() {
        selectors.push(parse_selector(pair));
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

fn parse_style_rule(pair: Pair<'_, Rule>) -> StylesheetRule {
    let mut iter = pair.into_inner();

    let selector = parse_selectors(iter.next().unwrap());
    let mut rules = Vec::new();

    for pair in iter {
        match pair.as_rule() {
            Rule::Attribute => {
                rules.push(parse_attribute(pair));
            }
            _ => unreachable!(),
        }
    }

    StylesheetRule {
        selectors: selector.into(),
        attributes: rules.into(),
    }
}

fn parse_style(input: &str) -> Result<Stylesheet, StyleheetParseError> {
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
    type Err = StyleheetParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        parse_style(input)
    }
}
