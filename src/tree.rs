use std::{
    collections::{HashMap, HashSet},
    hash::BuildHasherDefault,
    sync::atomic::{AtomicU64, Ordering},
};

use seahash::SeaHasher;

use crate::ViewId;

/// A context that tracks [`ViewId`]s.
pub trait Tracker {
    /// Get the underlying [`ViewId`] [`Tree`].
    ///
    /// This should only be used by platform implementation.
    fn tree(&mut self) -> &mut Tree;

    /// Register a [`ViewId`].
    fn register(&mut self, id: ViewId) {
        self.tree().register(id);
    }

    /// Unregister a [`ViewId`].
    fn unregister(&mut self, id: ViewId) {
        self.tree().unregister(id);
    }
}

/// Identifier of a node in a [`Tree`].
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId {
    id: u64,
}

impl NodeId {
    /// Get the next [`NodeId`].
    pub fn next() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        Self {
            id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
        }
    }
}

/// A tree of [`ViewId`]s.
///
/// This is an acceleration structure used to cull redundant branches when sending
/// [`Message`](crate::Message)s.
#[derive(Clone, Debug)]
pub struct Tree {
    nodes:   Vec<Node>,
    current: usize,
}

#[derive(Clone, Debug, Default)]
struct Node {
    parent: usize,
    nodes:  HashMap<NodeId, usize, BuildHasherDefault<SeaHasher>>,
    views:  HashSet<ViewId, BuildHasherDefault<SeaHasher>>,
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

impl Tree {
    /// Create new [`Tree`].
    pub fn new() -> Self {
        Self {
            nodes: vec![Node {
                parent: 0,
                nodes:  HashMap::default(),
                views:  HashSet::default(),
            }],

            current: 0,
        }
    }

    /// Add a child to the current node.
    pub fn insert(&mut self, id: NodeId) {
        // get the index of the new node
        let index = self.nodes.len();

        // insert the new node
        self.nodes.push(Node {
            parent: self.current,
            nodes:  HashMap::default(),
            views:  HashSet::default(),
        });

        let node = &mut self.nodes[self.current];

        if cfg!(debug_assertions) && node.nodes.contains_key(&id) {
            panic!("node with id {id:?} already exists");
        }

        // insert the new node as a child of the current
        node.nodes.insert(id, index);
    }

    /// Remove a child from the current node.
    pub fn remove(&mut self, id: NodeId) {
        self.nodes[self.current].nodes.remove(&id);
    }

    /// Swap two children of the current node.
    pub fn swap(&mut self, a: NodeId, b: NodeId) {
        if let Some(index_a) = self.nodes[self.current].nodes.remove(&a)
            && let Some(index_b) = self.nodes[self.current].nodes.remove(&b)
        {
            self.nodes[self.current].nodes.insert(a, index_b);
            self.nodes[self.current].nodes.insert(b, index_a);
        } else if cfg!(debug_assertions) {
            panic!("current node does not contain both {a:?} and {b:?}");
        }
    }

    /// Change the current node to a child of the current node.
    pub fn push(&mut self, id: NodeId) {
        if let Some(index) = self.nodes[self.current].nodes.get(&id) {
            self.current = *index;
        } else if cfg!(debug_assertions) {
            panic!("current node does not contain {id:?}");
        }
    }

    /// Change the current node to its parent.
    pub fn pop(&mut self) {
        if cfg!(debug_assertions) && self.current == 0 {
            panic!("the root cannot be popped");
        }

        self.current = self.nodes[self.current].parent;
    }

    /// Check whether the current node contains a [`ViewId`].
    pub fn contains(&mut self, id: ViewId) -> bool {
        self.nodes[self.current].views.contains(&id)
    }

    /// Register a [`ViewId`] in the current node.
    pub fn register(&mut self, id: ViewId) {
        let mut current = self.current;
        self.nodes[current].views.insert(id);

        while current != 0 {
            current = self.nodes[current].parent;
            self.nodes[current].views.insert(id);
        }
    }

    /// Unregister a [`ViewId`] from the current node.
    pub fn unregister(&mut self, id: ViewId) {
        let mut current = self.current;
        self.nodes[current].views.remove(&id);

        while current != 0 {
            current = self.nodes[current].parent;
            self.nodes[current].views.remove(&id);
        }
    }
}
