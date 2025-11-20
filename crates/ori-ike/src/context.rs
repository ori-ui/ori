use std::{
    any::{Any, TypeId},
    pin::Pin,
};

use winit::event_loop::EventLoopProxy;

pub struct Context {
    pub(crate) app:      ike::App,
    pub(crate) proxy:    EventLoopProxy<Event>,
    pub(crate) contexts: Vec<Box<dyn Any>>,
}

#[derive(Clone)]
pub struct Proxy {
    pub(crate) proxy: EventLoopProxy<Event>,
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
            proxy: self.proxy.clone(),
        }
    }
}

impl ori::ProviderContext for Context {
    fn push_context<T: Any>(&mut self, context: Box<T>) {
        self.contexts.push(Box::new(context))
    }

    fn pop_context<T: Any>(&mut self) -> Option<Box<T>> {
        self.contexts.pop()?.downcast().ok()
    }

    fn get_context<T: Any>(&mut self) -> Option<&T> {
        self.contexts
            .iter()
            .rfind(|c| c.as_ref().type_id() == TypeId::of::<T>())?
            .as_ref()
            .downcast_ref()
    }

    fn get_context_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.contexts
            .iter_mut()
            .rfind(|c| c.as_ref().type_id() == TypeId::of::<T>())?
            .as_mut()
            .downcast_mut()
    }
}

impl ori::Proxy for Proxy {
    fn rebuild(&self) {
        let _ = self.proxy.send_event(Event::Rebuild);
    }

    fn event(&self, event: ori::Event) {
        let _ = self.proxy.send_event(Event::Event(event));
    }

    fn spawn_boxed(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        let _ = self.proxy.send_event(Event::Spawn(future));
    }
}
