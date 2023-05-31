use std::sync::Arc;

use ori_reactive::OwnedSignal;

use crate::{AnyView, Element, ElementView, View};

enum NodeKind<V: ElementView> {
    Element(Element<V>),
    Fragment(Arc<[Node<V>]>),
    Dynamic(OwnedSignal<Node<V>>),
}

impl<V: ElementView> Clone for NodeKind<V> {
    fn clone(&self) -> Self {
        match self {
            Self::Element(element) => Self::Element(element.clone()),
            Self::Fragment(fragment) => Self::Fragment(fragment.clone()),
            Self::Dynamic(signal) => Self::Dynamic(signal.clone()),
        }
    }
}

/// A trait for types that can be converted into a [`Node`].
pub trait IntoNode<V: ElementView = Box<dyn AnyView>> {
    /// Converts `self` into a [`Node`].
    fn into_node(self) -> Node<V>;
}

impl<V: View> IntoNode<V> for V {
    fn into_node(self) -> Node<V> {
        Node::element(Element::new(self))
    }
}

impl<V: View> IntoNode for V {
    fn into_node(self) -> Node {
        Node::element(Element::new(self))
    }
}

impl<V: ElementView> IntoNode<V> for Node<V> {
    fn into_node(self) -> Node<V> {
        self
    }
}

impl<V: ElementView> IntoNode<V> for Element<V> {
    fn into_node(self) -> Node<V> {
        Node::element(self)
    }
}

/// A node in the UI tree.
///
/// A node can be one of the following:
/// - An [`Element`].
/// - A fragment of nodes.
/// - A dynamic node.
pub struct Node<V: ElementView = Box<dyn AnyView>> {
    kind: NodeKind<V>,
}

impl<V: ElementView> Clone for Node<V> {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind.clone(),
        }
    }
}

impl<V: ElementView> Node<V> {
    fn from_kind(kind: NodeKind<V>) -> Self {
        Self { kind }
    }

    /// Creates a new [`Node`].
    pub fn new(into_node: impl IntoNode<V>) -> Self {
        into_node.into_node()
    }

    /// Creates a new [`Node`] from an [`Element`].
    pub fn element(element: Element<V>) -> Self {
        Self::from_kind(NodeKind::Element(element))
    }

    /// Creates a new [`Node`] fragment from a list of [`Node`]s.
    pub fn fragment(fragment: impl Into<Arc<[Node<V>]>>) -> Self {
        Self::from_kind(NodeKind::Fragment(fragment.into()))
    }

    /// Creates a new dynamic [`Node`] from an [`OwnedSignal`].
    pub fn dynamic(signal: OwnedSignal<Node<V>>) -> Self {
        Self::from_kind(NodeKind::Dynamic(signal))
    }

    /// Creates an empty [`Node`].
    pub fn empty() -> Self {
        Self::fragment(Vec::new())
    }

    /// Returns the number of elements in the [`Node`].
    pub fn len(&self) -> usize {
        match &self.kind {
            NodeKind::Element(_) => 1,
            NodeKind::Fragment(fragment) => fragment.iter().map(Node::len).sum(),
            NodeKind::Dynamic(signal) => signal.get().len(),
        }
    }

    /// Returns `true` if the [`Node`] contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Tries to convert the [`Node`] into an [`Element`].
    pub fn into_element(self) -> Option<Element<V>> {
        match self.kind {
            NodeKind::Element(element) => Some(element),
            _ => None,
        }
    }

    /// Tries to convert the [`Node`] into a fragment of [`Node`]s.
    pub fn into_fragment(self) -> Option<Arc<[Node<V>]>> {
        match self.kind {
            NodeKind::Fragment(fragment) => Some(fragment),
            _ => None,
        }
    }

    /// Tries to convert the [`Node`] into a dynamic [`Node`].
    pub fn into_dynamic(self) -> Option<OwnedSignal<Node<V>>> {
        match self.kind {
            NodeKind::Dynamic(signal) => Some(signal),
            _ => None,
        }
    }

    /// Returns all elements in the [`Node`], including nested elements, flattened into a single
    /// [`Vec`]. Dynamic [`Node`]s are fetched in a reactive manner.
    pub fn flatten(&self) -> Vec<Element<V>> {
        match &self.kind {
            NodeKind::Element(element) => vec![element.clone()],
            NodeKind::Fragment(fragment) => fragment.iter().flat_map(Node::flatten).collect(),
            NodeKind::Dynamic(signal) => signal.get().flatten(),
        }
    }
}
