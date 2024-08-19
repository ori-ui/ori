//! Clipboard.

use std::fmt::Debug;

/// A clipboard.
pub struct Clipboard {
    backend: Box<dyn ClipboardBackend>,
}

impl Clipboard {
    /// Create a new clipboard from a backend.
    pub fn new(backend: Box<dyn ClipboardBackend>) -> Self {
        Self { backend }
    }

    /// Get the clipboard text.
    pub fn get(&mut self) -> String {
        self.backend.get_text()
    }

    /// Set the clipboard text.
    pub fn set(&mut self, text: impl AsRef<str>) {
        self.backend.set_text(text.as_ref());
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new(Box::new(NoopClipboard))
    }
}

impl Debug for Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Clipboard").finish()
    }
}

/// A clipboard backend.
pub trait ClipboardBackend {
    /// Get the clipboard text.
    fn get_text(&mut self) -> String;

    /// Set the clipboard text.
    fn set_text(&mut self, text: &str);
}

struct NoopClipboard;

impl ClipboardBackend for NoopClipboard {
    fn get_text(&mut self) -> String {
        String::new()
    }

    fn set_text(&mut self, _text: &str) {}
}
