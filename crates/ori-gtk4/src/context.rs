use std::{any::Any, path::PathBuf, pin::Pin};

use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use gtk4::gio::prelude::ApplicationExt as _;

#[derive(Clone)]
pub struct Context {
    app: gtk4::glib::WeakRef<gtk4::Application>,
    sender: UnboundedSender<Event>,
}

impl Context {
    pub fn new(
        app: gtk4::glib::WeakRef<gtk4::Application>,
    ) -> (Context, UnboundedReceiver<Event>) {
        let (sender, receiver) = unbounded();

        let context = Context { app, sender };

        (context, receiver)
    }

    pub fn app(&self) -> &gtk4::glib::WeakRef<gtk4::Application> {
        &self.app
    }

    pub fn event<T: Any + Send>(
        &self,
        item: T,
        target: impl Into<Option<ori::ViewId>>,
    ) {
        let event = ori::Event::new(item, target);

        self.sender.unbounded_send(Event::Event(event)).unwrap();
    }

    pub fn quit(&self) {
        if let Some(app) = self.app().upgrade() {
            app.quit();
        }
    }

    pub(crate) fn sender(&self) -> &UnboundedSender<Event> {
        &self.sender
    }
}

impl ori::AsyncContext for Context {
    type Proxy = Proxy;

    fn proxy(&mut self) -> Self::Proxy {
        Proxy {
            sender: self.sender.clone(),
        }
    }

    fn send_action(&mut self, action: ori::Action) {
        ori::Proxy::action(&self.proxy(), action);
    }
}

impl ori::Proxy for Proxy {
    fn rebuild(&self) {
        self.sender.unbounded_send(Event::Rebuild).unwrap();
    }

    fn event(&self, event: ori::Event) {
        self.sender.unbounded_send(Event::Event(event)).unwrap();
    }

    fn spawn_boxed(
        &self,
        future: Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
    ) {
        self.sender.unbounded_send(Event::Spawn(Box::pin(future))).unwrap();
    }
}

#[derive(Clone)]
pub struct Proxy {
    sender: UnboundedSender<Event>,
}

pub enum Event {
    CssChanged(PathBuf),

    Rebuild,
    Event(ori::Event),
    Spawn(Pin<Box<dyn Future<Output = ()> + Send>>),
}
