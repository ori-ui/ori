use std::{
    any::{Any, TypeId},
    pin::Pin,
    sync::mpsc::Sender,
};

use winit::event_loop::EventLoopProxy;

pub struct Context {
    pub(crate) app:     ike::App,
    pub(crate) proxy:   EventLoopProxy<()>,
    pub(crate) entries: Vec<Entry>,
    pub(crate) sender:  Sender<Event>,

    pub(crate) use_type_names_unsafe: bool,
}

pub(crate) struct Entry {
    value:     Box<dyn Any>,
    type_id:   TypeId,
    type_name: &'static str,
}

#[derive(Clone)]
pub struct Proxy {
    pub(crate) sender: Sender<Event>,
    pub(crate) proxy:  EventLoopProxy<()>,
}

pub enum Event {
    Rebuild,
    Event(ori::Event),
    Spawn(Pin<Box<dyn Future<Output = ()> + Send>>),
}

impl ike::BuildCx for Context {
    fn app(&self) -> &ike::App {
        &self.app
    }

    fn app_mut(&mut self) -> &mut ike::App {
        &mut self.app
    }
}

impl ori::BaseElement for Context {
    type Element = ike::WidgetId;
}

impl ori::AsyncContext for Context {
    type Proxy = Proxy;

    fn proxy(&mut self) -> Self::Proxy {
        Proxy {
            sender: self.sender.clone(),
            proxy:  self.proxy.clone(),
        }
    }
}

impl ori::ProviderContext for Context {
    fn push_context<T: Any>(&mut self, context: Box<T>) {
        self.entries.push(Entry {
            value:     context,
            type_id:   TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
        })
    }

    fn pop_context<T: Any>(&mut self) -> Option<Box<T>> {
        self.entries.pop()?.value.downcast().ok()
    }

    fn get_context<T: Any>(&self) -> Option<&T> {
        let entry = match self.use_type_names_unsafe {
            true => self
                .entries
                .iter()
                .rfind(|e| e.type_name == std::any::type_name::<T>())?,
            false => self
                .entries
                .iter()
                .rfind(|e| e.type_id == TypeId::of::<T>())?,
        };

        Some(unsafe { &*(entry.value.as_ref() as *const _ as *const T) })
    }

    fn get_context_mut<T: Any>(&mut self) -> Option<&mut T> {
        let entry = match self.use_type_names_unsafe {
            true => self
                .entries
                .iter_mut()
                .rfind(|e| e.type_name == std::any::type_name::<T>())?,
            false => self
                .entries
                .iter_mut()
                .rfind(|e| e.type_id == TypeId::of::<T>())?,
        };

        Some(unsafe { &mut *(entry.value.as_mut() as *mut _ as *mut T) })
    }
}

impl ori::Proxy for Proxy {
    fn rebuild(&self) {
        let _ = self.sender.send(Event::Rebuild);
        let _ = self.proxy.send_event(());
    }

    fn event(&self, event: ori::Event) {
        let _ = self.sender.send(Event::Event(event));
        let _ = self.proxy.send_event(());
    }

    fn spawn_boxed(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        let _ = self.sender.send(Event::Spawn(future));
        let _ = self.proxy.send_event(());
    }
}
