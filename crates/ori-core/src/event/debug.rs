use std::{
    mem,
    sync::{Mutex, MutexGuard},
};

use crate::{DebugNode, Event, EventContext, Node, View};

#[derive(Debug, Default)]
pub struct DebugEvent {
    node: Mutex<DebugNode>,
}

impl DebugEvent {
    pub fn new(node: DebugNode) -> Self {
        Self {
            node: Mutex::new(node),
        }
    }

    pub fn take(&self) -> DebugNode {
        mem::take(&mut self.node.lock().unwrap())
    }

    pub fn node(&self) -> MutexGuard<DebugNode> {
        self.node.lock().unwrap()
    }

    pub fn add_child(&self, child: DebugNode) {
        self.node.lock().unwrap().children.push(child);
    }

    /// Sets the root node of the current debug tree.
    pub fn set_node<T: View>(&self, cx: &mut EventContext, node: &Node<T>) {
        let debug_node = DebugNode {
            selectors: cx.selectors.clone(),
            local_rect: node.local_rect(),
            global_rect: node.global_rect(),
            children: Vec::new(),
        };

        *self.node.lock().unwrap() = debug_node;
    }

    /// This method is used to add a child node to the debug tree.
    ///
    /// This will call the `event` method.
    pub fn with_node<T: View>(&self, cx: &mut EventContext, node: &Node<T>) {
        let debug_node = DebugNode {
            selectors: cx.selectors.clone(),
            local_rect: node.local_rect(),
            global_rect: node.global_rect(),
            children: Vec::new(),
        };

        let event = Event::new(DebugEvent::new(debug_node));
        node.view().event(&mut node.view_state(), cx, &event);

        let child = event.get::<DebugEvent>().unwrap().take();
        self.add_child(child);
    }
}
