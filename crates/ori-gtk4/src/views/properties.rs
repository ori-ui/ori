use crate::{Context, View};

pub trait IntoCssClasses<I> {
    fn into_css_classes(self) -> Vec<String>;
}

impl<T> IntoCssClasses<T> for T
where
    T: ToString,
{
    fn into_css_classes(self) -> Vec<String> {
        self.to_string()
            .split_whitespace()
            .map(String::from)
            .collect()
    }
}

struct IterImpl;

impl<I> IntoCssClasses<IterImpl> for I
where
    I: IntoIterator<Item: ToString>,
{
    fn into_css_classes(self) -> Vec<String> {
        self.into_iter()
            .flat_map(IntoCssClasses::into_css_classes)
            .collect()
    }
}

pub trait WithProp: Sized {
    fn can_focus(self, can_focus: bool) -> Prop<Self> {
        Prop::new(self, Property::CanFocus(can_focus))
    }

    fn can_target(self, can_target: bool) -> Prop<Self> {
        Prop::new(self, Property::CanTarget(can_target))
    }

    fn css_class<I>(self, css_classes: impl IntoCssClasses<I>) -> Prop<Self> {
        let property = Property::CssClasses(css_classes.into_css_classes());
        Prop::new(self, property)
    }

    fn halign(self, align: Align) -> Prop<Self> {
        Prop::new(self, Property::Halign(align))
    }

    fn valign(self, align: Align) -> Prop<Self> {
        Prop::new(self, Property::Valign(align))
    }

    fn width_request(self, request: impl Into<Option<u32>>) -> Prop<Self> {
        let property = Property::WidthRequest(request.into());
        Prop::new(self, property)
    }

    fn height_request(self, request: impl Into<Option<u32>>) -> Prop<Self> {
        let property = Property::HeightRequest(request.into());
        Prop::new(self, property)
    }

    fn opacity(self, opacity: f32) -> Prop<Self> {
        Prop::new(self, Property::Opacity(opacity))
    }

    fn tooltip<U: ToString>(self, tooltip: impl Into<Option<U>>) -> Prop<Self> {
        let tooltip = tooltip.into().as_ref().map(ToString::to_string);
        Prop::new(self, Property::Tooltip(tooltip))
    }

    fn hexpand(self, expand: impl Into<Option<bool>>) -> Prop<Self> {
        Prop::new(self, Property::Hexpand(expand.into()))
    }

    fn vexpand(self, expand: impl Into<Option<bool>>) -> Prop<Self> {
        Prop::new(self, Property::Vexpand(expand.into()))
    }

    fn visible(self, visible: bool) -> Prop<Self> {
        Prop::new(self, Property::Visible(visible))
    }
}

impl<V> WithProp for V {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Align {
    Start,
    Center,
    End,
    Fill,
    Baseline,
}

pub struct Prop<V> {
    content: V,
    property: Property,
}

#[derive(PartialEq)]
pub enum Property {
    CanFocus(bool),
    CanTarget(bool),
    CssClasses(Vec<String>),
    Focusable(bool),
    Halign(Align),
    Valign(Align),
    WidthRequest(Option<u32>),
    HeightRequest(Option<u32>),
    Opacity(f32),
    Tooltip(Option<String>),
    Hexpand(Option<bool>),
    Vexpand(Option<bool>),
    Visible(bool),
}

impl<V> Prop<V> {
    fn new(content: V, property: Property) -> Self {
        Self { content, property }
    }
}

impl<T, V> ori::View<Context, T> for Prop<V>
where
    V: View<T>,
{
    type Element = V::Element;
    type State = (Property, V::State);

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (element, state) = self.content.build(cx, data);

        // record the current state of the property and apply it
        let prev = self.property.get(&element);
        self.property.set(&element);

        (element, (prev, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (prev, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        old: &mut Self,
    ) -> bool {
        // since pretty much anything can happen to the element, imagine a view dynamically
        // swapping between two elements, we have to restore the property to it's original state
        // before rebuilding.
        prev.set(element);

        let changed = self.content.rebuild(
            element,
            state,
            cx,
            data,
            &mut old.content,
        );

        // we now record the state of the new property and apply it
        *prev = self.property.get(element);
        self.property.set(element);

        changed
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (_, state): Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.content.teardown(element, state, cx, data);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (prev, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> (bool, ori::Action) {
        // same idea as rebuild
        prev.set(element);

        let (changed, action) = self.content.event(element, state, cx, data, event);

        *prev = self.property.get(element);
        self.property.set(element);

        (changed, action)
    }
}

mod imp {
    use gtk4::{glib::object::IsA, prelude::WidgetExt};

    use super::{Align, Property};

    impl Property {
        pub(super) fn set(&self, element: &impl IsA<gtk4::Widget>) {
            match *self {
                Property::CanFocus(can_focus) => {
                    element.set_can_focus(can_focus);
                }

                Property::CanTarget(can_target) => {
                    element.set_can_target(can_target);
                }

                Property::CssClasses(ref classes) => {
                    for class in classes {
                        element.add_css_class(class);
                    }
                }

                Property::Focusable(focusable) => {
                    element.set_focusable(focusable);
                }

                Property::Halign(align) => {
                    element.set_halign(align_to_gtk(align));
                }

                Property::Valign(align) => {
                    element.set_valign(align_to_gtk(align));
                }

                Property::WidthRequest(request) => {
                    element.set_width_request(request.map_or(-1, |r| r as i32));
                }

                Property::HeightRequest(request) => {
                    element.set_height_request(request.map_or(-1, |r| r as i32));
                }

                Property::Opacity(opacity) => {
                    element.set_opacity(opacity as f64);
                }

                Property::Tooltip(ref tooltip) => {
                    element.set_tooltip_text(tooltip.as_deref());
                }

                Property::Hexpand(expand) => {
                    if let Some(expand) = expand {
                        element.set_hexpand(expand);
                    }

                    element.set_hexpand_set(expand.is_some());
                }

                Property::Vexpand(expand) => {
                    if let Some(expand) = expand {
                        element.set_vexpand(expand);
                    }

                    element.set_vexpand_set(expand.is_some());
                }

                Property::Visible(visible) => {
                    element.set_visible(visible);
                }
            }
        }

        pub(super) fn get(&self, element: &impl IsA<gtk4::Widget>) -> Self {
            match self {
                Property::CanFocus(_) => Property::CanFocus(element.can_focus()),
                Property::CanTarget(_) => Property::CanTarget(element.can_target()),
                Property::CssClasses(_) => Property::CssClasses(
                    element.css_classes().into_iter().map(Into::into).collect(),
                ),

                Property::Focusable(_) => Property::Focusable(element.is_focusable()),

                Property::Halign(_) => Property::Halign(gtk_to_align(element.halign())),

                Property::Valign(_) => Property::Valign(gtk_to_align(element.valign())),

                Property::WidthRequest(_) => match element.width_request() {
                    request if request < 0 => Property::WidthRequest(None),
                    request => Property::WidthRequest(Some(request as u32)),
                },

                Property::HeightRequest(_) => match element.height_request() {
                    request if request < 0 => Property::HeightRequest(None),
                    request => Property::HeightRequest(Some(request as u32)),
                },

                Property::Opacity(_) => Property::Opacity(element.opacity() as f32),

                Property::Tooltip(_) => Property::Tooltip(element.tooltip_text().map(Into::into)),

                Property::Hexpand(_) => match element.is_hexpand_set() {
                    true => Property::Hexpand(Some(element.hexpands())),
                    false => Property::Hexpand(None),
                },

                Property::Vexpand(_) => match element.is_vexpand_set() {
                    true => Property::Vexpand(Some(element.vexpands())),
                    false => Property::Vexpand(None),
                },

                Property::Visible(_) => Property::Visible(element.is_visible()),
            }
        }
    }

    fn align_to_gtk(align: Align) -> gtk4::Align {
        match align {
            Align::Start => gtk4::Align::Start,
            Align::Center => gtk4::Align::Center,
            Align::End => gtk4::Align::End,
            Align::Fill => gtk4::Align::Fill,
            Align::Baseline => gtk4::Align::Baseline,
        }
    }

    fn gtk_to_align(gtk: gtk4::Align) -> Align {
        match gtk {
            gtk4::Align::Start => Align::Start,
            gtk4::Align::Center => Align::Center,
            gtk4::Align::End => Align::End,
            gtk4::Align::Fill => Align::Fill,
            gtk4::Align::Baseline => Align::Baseline,
            _ => Align::Fill,
        }
    }
}
