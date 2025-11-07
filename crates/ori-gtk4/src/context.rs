use std::{any::Any, path::PathBuf};

use gtk4::gio::prelude::ApplicationExt as _;
use ori::Action;
use tokio::sync::mpsc::{
    UnboundedReceiver, UnboundedSender, unbounded_channel,
};

#[derive(Clone)]
pub struct Context {
    app: gtk4::glib::WeakRef<gtk4::Application>,
    sender: UnboundedSender<Event>,
}

impl Context {
    pub fn new(
        app: gtk4::glib::WeakRef<gtk4::Application>,
    ) -> (Context, UnboundedReceiver<Event>) {
        let (sender, receiver) = unbounded_channel();

        let context = Context { app, sender };

        (context, receiver)
    }

    pub fn app(&self) -> &gtk4::glib::WeakRef<gtk4::Application> {
        &self.app
    }

    pub fn event<T: Any + Send + Sync>(
        &self,
        item: T,
        target: impl Into<Option<ori::ViewId>>,
    ) {
        let event = ori::Event::new(item, target);

        self.sender.send(Event::Event(event)).expect("channel not closed");
    }

    pub fn rebuild(&self) {
        self.sender.send(Event::Rebuild).expect("channel not closed");
    }

    pub fn quit(&self) {
        if let Some(app) = self.app().upgrade() {
            app.quit();
        }
    }

    pub(crate) fn activate(&self) {
        self.sender.send(Event::Activate).expect("channel not closed");
    }

    pub(crate) fn opened(&self, id: u64) {
        self.sender
            .send(Event::InitialWindowCreated(id))
            .expect("channel not closed");
    }

    pub(crate) fn closed(&self, window_id: u64) {
        self.sender
            .send(Event::WindowClosed(window_id))
            .expect("channel not closed");
    }

    pub(crate) fn sender(&self) -> &UnboundedSender<Event> {
        &self.sender
    }
}

impl ori::Context for Context {
    fn action(&mut self, action: Action) {
        self.sender.send(Event::Action(action)).expect("channel not closed");
    }
}

pub enum Event {
    Activate,
    Rebuild,
    InitialWindowCreated(u64),
    WindowClosed(u64),
    CssChanged(PathBuf),
    Event(ori::Event),
    Action(Action),
}
