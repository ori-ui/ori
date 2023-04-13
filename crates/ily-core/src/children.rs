use deref_derive::{Deref, DerefMut};

use crate::{Node, View};

pub trait Parent {
    fn add_child(&mut self, child: impl View);
}

#[derive(Default, Deref, DerefMut)]
pub struct Children {
    nodes: Vec<Node>,
}

impl Children {
    pub const fn new() -> Self {
        Self { nodes: Vec::new() }
    }
}

impl IntoIterator for Children {
    type Item = Node;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl<'a> IntoIterator for &'a Children {
    type Item = &'a Node;
    type IntoIter = std::slice::Iter<'a, Node>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

impl<'a> IntoIterator for &'a mut Children {
    type Item = &'a mut Node;
    type IntoIter = std::slice::IterMut<'a, Node>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter_mut()
    }
}
