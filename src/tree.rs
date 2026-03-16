use std::{collections::HashSet, fmt, hash::BuildHasherDefault};

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

/// A tree of [`ViewId`]s.
///
/// This is an acceleration structure used to cull redundant branches when sending
/// [`Message`](crate::Message)s.
#[derive(Clone, Debug, Default)]
pub struct Tree {
    root:  Node,
    stack: Vec<usize>,
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn recurse(f: &mut fmt::Formatter<'_>, node: &Node, indent: usize) -> fmt::Result {
            write!(
                f,
                "{}node: {}",
                "  ".repeat(indent),
                node.views
                    .iter()
                    .map(|id| format!("v{id:?}"))
                    .collect::<Vec<_>>()
                    .join(", "),
            )?;

            for node in &node.nodes {
                writeln!(f)?;
                recurse(f, node, indent + 1)?;
            }

            Ok(())
        }

        writeln!(f, "tree:")?;
        recurse(f, &self.root, 1)
    }
}

#[derive(Clone, Debug, Default)]
struct Node {
    nodes: Vec<Node>,
    views: HashSet<ViewId, BuildHasherDefault<SeaHasher>>,
}

impl Node {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            views: HashSet::default(),
        }
    }
}

impl Tree {
    /// Create a new [`Tree`].
    pub fn new() -> Self {
        Self {
            root:  Node::new(),
            stack: vec![0],
        }
    }

    /// Reset the state of the [`Tree`].
    ///
    /// The platform implementation will call this before calling methods on the root
    /// [`View`](crate::View).
    pub fn reset(&mut self) {
        self.stack.clear();
        self.stack.push(0);
    }

    pub(crate) fn push(&mut self) {
        self.stack.push(0);
    }

    pub(crate) fn pop(&mut self) {
        self.stack.pop();

        if let Some(index) = self.stack.last_mut() {
            *index += 1;
        }
    }

    pub(crate) fn insert(&mut self) {
        let index = self
            .stack
            .last()
            .copied()
            .expect("these is always one element in the stack");

        self.current_mut().nodes.insert(index, Node::new());
    }

    pub(crate) fn remove(&mut self) {
        let index = self
            .stack
            .last_mut()
            .expect("these is always one element in the stack");

        *index -= 1;
        let index = *index;

        self.current_mut().nodes.remove(index);
    }

    pub(crate) fn swap(&mut self, offset: usize) {
        let index = self
            .stack
            .last()
            .copied()
            .expect("these is always one element in the stack");

        self.current_mut().nodes.swap(index, index + offset);
    }

    pub(crate) fn contains(&self, id: ViewId) -> bool {
        self.current().views.contains(&id)
    }

    fn register(&mut self, id: ViewId) {
        let mut node = &mut self.root;

        for index in self.stack.iter().take(self.stack.len() - 1).copied() {
            node.views.insert(id);
            node = &mut node.nodes[index];
        }

        node.views.insert(id);
    }

    fn unregister(&mut self, id: ViewId) {
        let mut node = &mut self.root;

        for index in self.stack.iter().take(self.stack.len() - 1).copied() {
            node.views.remove(&id);
            node = &mut node.nodes[index];
        }

        node.views.remove(&id);
    }

    fn current(&self) -> &Node {
        let mut node = &self.root;

        for index in self.stack.iter().take(self.stack.len() - 1).copied() {
            node = &node.nodes[index];
        }

        node
    }

    fn current_mut(&mut self) -> &mut Node {
        let mut node = &mut self.root;

        for index in self.stack.iter().take(self.stack.len() - 1).copied() {
            node = &mut node.nodes[index];
        }

        node
    }
}
