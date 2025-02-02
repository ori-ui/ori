use std::{
    any::{self, Any},
    future::Future,
};

use crate::{
    clipboard::Clipboard,
    command::{Command, CommandProxy},
    layout::Size,
    text::{Fonts, Paragraph, TextLayoutLine},
};

use super::Contexts;

/// A base context that is shared between all other contexts.
pub struct BaseCx<'a> {
    pub(crate) contexts: &'a mut Contexts,
    pub(crate) proxy: &'a mut CommandProxy,
}

impl<'a> BaseCx<'a> {
    /// Create a new base context.
    pub fn new(contexts: &'a mut Contexts, proxy: &'a mut CommandProxy) -> Self {
        Self { contexts, proxy }
    }

    /// Get the [`Fonts`] as a mutable reference.
    pub fn fonts(&mut self) -> &mut dyn Fonts {
        self.context_mut::<Box<dyn Fonts>>().as_mut()
    }

    /// Measure a paragraph.
    pub fn measure_paragraph(&mut self, paragraph: &Paragraph, width: f32) -> Size {
        self.fonts().measure(paragraph, width)
    }

    /// Layout a paragraph.
    pub fn layout_paragraph(&mut self, paragraph: &Paragraph, width: f32) -> Vec<TextLayoutLine> {
        self.fonts().layout(paragraph, width)
    }

    /// Get the [`Clipboard`].
    pub fn clipboard(&mut self) -> &mut Clipboard {
        self.context_or_default::<Clipboard>()
    }

    /// Get the [`CommandProxy`].
    pub fn proxy(&self) -> CommandProxy {
        self.proxy.clone()
    }

    /// Emit a command.
    pub fn cmd<T: Any + Send>(&mut self, command: T) {
        self.proxy.cmd_silent(Command::new(command));
    }

    /// Spawn a future.
    pub fn spawn_async(&mut self, future: impl Future<Output = ()> + Send + 'static) {
        self.proxy.spawn_async(future);
    }

    /// Spawn a future sending a command when it completes.
    pub fn cmd_async<T: Any + Send>(&self, future: impl Future<Output = T> + Send + 'static) {
        self.proxy.cmd_async(future);
    }

    /// Get a reference to the [`Contexts`].
    pub fn contexts(&self) -> &Contexts {
        self.contexts
    }

    /// Get a mutable reference to the [`Contexts`].
    pub fn contexts_mut(&mut self) -> &mut Contexts {
        self.contexts
    }

    /// Insert a context.
    pub fn insert_context<T: Any>(&mut self, context: T) -> Option<T> {
        self.contexts.insert(context)
    }

    /// Check if a context is contained.
    pub fn contains_context<T: Any>(&self) -> bool {
        self.contexts.contains::<T>()
    }

    /// Remove a context.
    pub fn remove_context<T: Any>(&mut self) -> Option<T> {
        self.contexts.remove::<T>()
    }

    /// Get a context.
    #[inline(always)]
    pub fn get_context<T: Any>(&self) -> Option<&T> {
        self.contexts.get::<T>()
    }

    /// Get a mutable context.
    #[inline(always)]
    pub fn get_context_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.contexts.get_mut::<T>()
    }

    /// Get a context.
    ///
    /// # Panics
    /// - If the context is not found.
    #[track_caller]
    #[inline(always)]
    pub fn context<T: Any>(&self) -> &T {
        match self.get_context::<T>() {
            Some(context) => context,
            None => panic!("context not found: {}", any::type_name::<T>()),
        }
    }

    /// Get a mutable context.
    ///
    /// # Panics
    /// - If the context is not found.
    #[track_caller]
    #[inline(always)]
    pub fn context_mut<T: Any>(&mut self) -> &mut T {
        match self.get_context_mut::<T>() {
            Some(context) => context,
            None => panic!("context not found: {}", any::type_name::<T>()),
        }
    }

    /// Get a context or insert a `default`.
    pub fn context_or_default<T: Any + Default>(&mut self) -> &mut T {
        self.contexts.get_or_default::<T>()
    }
}
