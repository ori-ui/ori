use std::{
    collections::{linked_list, LinkedList},
    fmt::Debug,
};

use crate::window::{WindowDescriptor, WindowId};

use super::UiBuilder;

/// Requests the [`Ui`](super::Ui) might make to the application shell.
#[must_use]
pub enum UiRequest<T> {
    /// Render a window.
    Render(WindowId),
    /// Create a window.
    CreateWindow(WindowDescriptor, UiBuilder<T>),
    /// Remove a window.
    RemoveWindow(WindowId),
}

impl<T> Debug for UiRequest<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UiRequest::Render(id) => write!(f, "Render({:?})", id),
            UiRequest::CreateWindow(desc, _) => write!(f, "CreateWindow({:?})", desc),
            UiRequest::RemoveWindow(id) => write!(f, "RemoveWindow({:?})", id),
        }
    }
}

/// A list of [`UiRequest`]s.
#[must_use]
pub struct UiRequests<T> {
    requests: LinkedList<UiRequest<T>>,
}

impl<T> UiRequests<T> {
    /// Create a new empty list of requests.
    pub fn new() -> Self {
        Self {
            requests: LinkedList::new(),
        }
    }

    /// Get the number of requests in the list.
    pub fn len(&self) -> usize {
        self.requests.len()
    }

    /// Get `true` if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }

    /// Push a request to the list.
    pub fn push(&mut self, request: UiRequest<T>) {
        self.requests.push_back(request);
    }

    /// Extend the list with the given `requests`.
    pub fn extend(&mut self, mut requests: UiRequests<T>) {
        self.requests.append(&mut requests.requests);
    }

    /// Returns the next request in the list.
    pub fn pop(&mut self) -> Option<UiRequest<T>> {
        self.requests.pop_front()
    }
}

impl<T> Default for UiRequests<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> IntoIterator for UiRequests<T> {
    type Item = UiRequest<T>;
    type IntoIter = linked_list::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.requests.into_iter()
    }
}

impl<T> Debug for UiRequests<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.requests.iter()).finish()
    }
}
