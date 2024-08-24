use std::sync::{mpsc::Receiver, mpsc::Sender, Arc, Mutex};

use ori_core::clipboard::ClipboardBackend;
use x11rb::{
    connection::Connection as _,
    protocol::{
        xproto::{
            AtomEnum, ConnectionExt as _, CreateWindowAux, EventMask, PropMode,
            SelectionNotifyEvent, WindowClass,
        },
        Event as XEvent,
    },
    wrapper::ConnectionExt as _,
    xcb_ffi::XCBConnection,
};

use super::{launch::Atoms, X11Error};

pub struct X11ClipboardServer {
    owner: u32,
    atoms: Atoms,
    sender: Sender<String>,
    data: Arc<Mutex<String>>,
}

impl X11ClipboardServer {
    pub fn new(
        conn: &Arc<XCBConnection>,
        atoms: Atoms,
    ) -> Result<(X11ClipboardServer, X11Clipboard), X11Error> {
        let screen = &conn.setup().roots[0];
        let owner = conn.generate_id()?;
        conn.create_window(
            x11rb::COPY_DEPTH_FROM_PARENT,
            owner,
            screen.root,
            0,
            0,
            1,
            1,
            0,
            WindowClass::INPUT_OUTPUT,
            x11rb::COPY_FROM_PARENT,
            &CreateWindowAux::new(),
        )?
        .check()?;

        let (sender, receiver) = std::sync::mpsc::channel();
        let data = Arc::new(Mutex::new(String::new()));

        let server = X11ClipboardServer {
            owner,
            atoms,
            sender,
            data: data.clone(),
        };

        let clipboard = X11Clipboard {
            conn: conn.clone(),
            owner,
            atoms,
            receiver,
            data,
        };

        Ok((server, clipboard))
    }

    pub fn handle_event(&self, conn: &XCBConnection, event: &XEvent) -> Result<(), X11Error> {
        match event {
            XEvent::SelectionNotify(event) => {
                if event.selection != self.atoms.CLIPBOARD {
                    unreachable!();
                }

                if event.property == x11rb::NONE {
                    // FIXME: conversion failed, handle this!
                    return Ok(());
                }

                let reply = conn.get_property(
                    false,
                    self.owner,
                    event.property,
                    self.atoms.UTF8_STRING,
                    0,
                    u32::MAX,
                )?;

                let data = reply.reply()?;
                let data = data.value8().into_iter().flatten().collect::<Vec<_>>();

                let text = String::from_utf8(data).unwrap();
                let _ = self.sender.send(text);

                Ok(())
            }
            XEvent::SelectionRequest(event) => {
                if event.target == self.atoms.TARGETS {
                    conn.change_property32(
                        PropMode::REPLACE,
                        event.requestor,
                        event.property,
                        AtomEnum::ATOM,
                        &[self.atoms.TARGETS, self.atoms.UTF8_STRING],
                    )?;
                } else {
                    conn.change_property8(
                        PropMode::REPLACE,
                        event.requestor,
                        event.property,
                        self.atoms.UTF8_STRING,
                        self.data.lock().unwrap().as_bytes(),
                    )?;
                }

                conn.send_event(
                    false,
                    event.requestor,
                    EventMask::NO_EVENT,
                    SelectionNotifyEvent {
                        response_type: x11rb::protocol::xproto::SELECTION_NOTIFY_EVENT,
                        sequence: 0,
                        time: event.time,
                        requestor: event.requestor,
                        selection: event.selection,
                        target: event.target,
                        property: event.property,
                    },
                )?;
                conn.flush()?;

                Ok(())
            }
            _ => Ok(()),
        }
    }
}

pub struct X11Clipboard {
    conn: Arc<XCBConnection>,
    owner: u32,
    atoms: Atoms,
    receiver: Receiver<String>,
    data: Arc<Mutex<String>>,
}

impl X11Clipboard {
    fn request(&self) -> Result<(), X11Error> {
        self.conn.convert_selection(
            self.owner,
            self.atoms.CLIPBOARD,
            self.atoms.UTF8_STRING,
            self.atoms.XSEL_DATA,
            x11rb::CURRENT_TIME,
        )?;
        self.conn.flush()?;

        Ok(())
    }

    fn set_text(&mut self, text: &str) -> Result<(), X11Error> {
        *self.data.lock().unwrap() = text.to_string();
        (self.conn).set_selection_owner(self.owner, self.atoms.CLIPBOARD, x11rb::CURRENT_TIME)?;
        self.conn.flush()?;
        Ok(())
    }
}

impl ClipboardBackend for X11Clipboard {
    fn get_text(&mut self) -> String {
        self.request().unwrap();
        self.receiver.recv().unwrap()
    }

    fn set_text(&mut self, text: &str) {
        self.set_text(text).unwrap();
    }
}
