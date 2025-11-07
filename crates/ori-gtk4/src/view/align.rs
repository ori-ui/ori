use gtk4::prelude::WidgetExt as _;

use crate::{Context, View};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Align {
    Start,
    Center,
    End,
    Fill,
    Baseline,
}

impl From<Align> for gtk4::Align {
    fn from(alignment: Align) -> Self {
        match alignment {
            Align::Start => gtk4::Align::Start,
            Align::Center => gtk4::Align::Center,
            Align::End => gtk4::Align::End,
            Align::Fill => gtk4::Align::Fill,
            Align::Baseline => gtk4::Align::Baseline,
        }
    }
}

pub fn align<V>(content: V) -> Alignment<V> {
    Alignment::new(content)
}

pub fn halign<V>(align: Align, content: V) -> Alignment<V> {
    Alignment::new(content).halign(align)
}

pub fn valign<V>(align: Align, content: V) -> Alignment<V> {
    Alignment::new(content).valign(align)
}

pub fn center<V>(content: V) -> Alignment<V> {
    Alignment::new(content).halign(Align::Center).valign(Align::Center)
}

#[must_use]
pub struct Alignment<V> {
    pub content: V,
    pub halign: Option<Align>,
    pub valign: Option<Align>,
}

impl<V> Alignment<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            halign: None,
            valign: None,
        }
    }

    pub fn halign(mut self, align: Align) -> Self {
        self.halign = Some(align);
        self
    }

    pub fn valign(mut self, align: Align) -> Self {
        self.valign = Some(align);
        self
    }
}

impl<T, V: View<T>> ori::View<Context, T> for Alignment<V> {
    type Element = V::Element;
    type State = V::State;

    fn build(
        &mut self,
        cx: &mut Context,
        data: &mut T,
    ) -> (Self::Element, Self::State) {
        let (element, state) = self.content.build(cx, data);

        if let Some(halign) = self.halign {
            element.set_halign(halign.into());
        }

        if let Some(valign) = self.valign {
            element.set_valign(valign.into());
        }

        (element, state)
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        old: &mut Self,
    ) {
        self.content.rebuild(
            element,
            state,
            cx,
            data,
            &mut old.content,
        );

        if let Some(halign) = self.halign {
            element.set_halign(halign.into());
        }

        if let Some(valign) = self.valign {
            element.set_valign(valign.into());
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        state: Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.content.teardown(element, state, cx, data);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        self.content.event(element, state, cx, data, event)
    }
}
