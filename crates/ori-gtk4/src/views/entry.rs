use gtk4::{
    glib::object::ObjectExt as _,
    prelude::{EditableExt as _, EntryExt as _},
};

use crate::Context;

pub fn entry<T>() -> Entry<T> {
    Entry::new()
}

enum EntryEvent {
    Change(String),
    Submit(String),
}

#[must_use]
pub struct Entry<T> {
    pub text:        Option<String>,
    pub placeholder: Option<String>,
    pub on_change:   Box<dyn FnMut(&mut T, String) -> ori::Action>,
    pub on_submit:   Box<dyn FnMut(&mut T, String) -> ori::Action>,
}

impl<T> Default for Entry<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Entry<T> {
    pub fn new() -> Self {
        Self {
            text:        None,
            placeholder: None,
            on_change:   Box::new(|_, _| ori::Action::new()),
            on_submit:   Box::new(|_, _| ori::Action::new()),
        }
    }

    pub fn text(mut self, text: impl ToString) -> Self {
        self.text = Some(text.to_string());
        self
    }

    pub fn placeholder(mut self, placeholder: impl ToString) -> Self {
        self.placeholder = Some(placeholder.to_string());
        self
    }

    pub fn on_change<A>(mut self, mut on_change: impl FnMut(&mut T, String) -> A + 'static) -> Self
    where
        A: ori::IntoAction,
    {
        self.on_change = Box::new(move |d, t| on_change(d, t).into_action());
        self
    }

    pub fn on_submit<A>(mut self, mut on_submit: impl FnMut(&mut T, String) -> A + 'static) -> Self
    where
        A: ori::IntoAction,
    {
        self.on_submit = Box::new(move |d, t| on_submit(d, t).into_action());
        self
    }
}

impl<T> ori::ViewMarker for Entry<T> {}
impl<T> ori::View<Context, T> for Entry<T> {
    type Element = gtk4::Entry;
    type State = (ori::Key, gtk4::glib::SignalHandlerId);

    fn build(&mut self, cx: &mut Context, _data: &mut T) -> (Self::Element, Self::State) {
        let element = gtk4::Entry::new();

        if let Some(ref text) = self.text {
            element.set_text(text);
        }

        element.set_placeholder_text(self.placeholder.as_deref());

        let id = ori::Key::next();

        let changed = element.connect_changed({
            let cx = cx.clone();
            move |text| {
                let event = EntryEvent::Change(text.text().into());
                cx.event(event, id);
            }
        });

        element.connect_activate({
            let cx = cx.clone();
            move |text| {
                let event = EntryEvent::Submit(text.text().into());
                cx.event(event, id);
            }
        });

        (element, (id, changed))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (_id, changed): &mut Self::State,
        _cx: &mut Context,
        _data: &mut T,
        old: &mut Self,
    ) {
        if self.text != old.text
            && let Some(ref text) = self.text
            && **text != element.text()
        {
            element.block_signal(changed);
            element.set_text(text);
            element.unblock_signal(changed);
        }

        if self.placeholder != old.placeholder {
            element.set_placeholder_text(self.placeholder.as_deref());
        }
    }

    fn teardown(
        &mut self,
        _element: Self::Element,
        _id: Self::State,
        _cx: &mut Context,
        _data: &mut T,
    ) {
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        (id, _changed): &mut Self::State,
        _cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        match event.take_targeted(*id) {
            Some(EntryEvent::Change(text)) => (self.on_change)(data, text),
            Some(EntryEvent::Submit(text)) => (self.on_submit)(data, text),
            None => ori::Action::new(),
        }
    }
}
