use std::{
    mem,
    sync::{Mutex, MutexGuard},
};

use ori_reactive::Event;

use crate::{DebugElement, Element, ElementView, EventContext};

#[derive(Debug, Default)]
pub struct DebugEvent {
    element: Mutex<DebugElement>,
}

impl DebugEvent {
    pub fn new(element: DebugElement) -> Self {
        Self {
            element: Mutex::new(element),
        }
    }

    pub fn take(&self) -> DebugElement {
        mem::take(&mut self.element.lock().unwrap())
    }

    pub fn element(&self) -> MutexGuard<DebugElement> {
        self.element.lock().unwrap()
    }

    pub fn add_child(&self, child: DebugElement) {
        self.element.lock().unwrap().children.push(child);
    }

    /// Sets the root element of the current debug tree.
    pub fn set_element<T: ElementView>(&self, cx: &mut EventContext, element: &Element<T>) {
        let debug_element = DebugElement {
            selectors: cx.selectors.clone(),
            local_rect: element.local_rect(),
            global_rect: element.global_rect(),
            children: Vec::new(),
        };

        *self.element.lock().unwrap() = debug_element;
    }

    /// This method is used to add a child element to the debug tree.
    ///
    /// This will call the `event` method.
    pub fn with_element<T: ElementView>(&self, cx: &mut EventContext, element: &Element<T>) {
        let debug_element = DebugElement {
            selectors: cx.selectors.clone(),
            local_rect: element.local_rect(),
            global_rect: element.global_rect(),
            children: Vec::new(),
        };

        let event = Event::new(DebugEvent::new(debug_element));
        element.view().event(&mut element.view_state(), cx, &event);

        let child = event.get::<DebugEvent>().unwrap().take();
        self.add_child(child);
    }
}
