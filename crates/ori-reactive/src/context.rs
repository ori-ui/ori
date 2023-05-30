use std::{any::Any, fmt::Debug, sync::Arc};

#[derive(Clone, Default)]
pub(crate) struct Contexts {
    contexts: Vec<Arc<dyn Any + Send + Sync>>,
}

impl Debug for Contexts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Contexts")
            .field("len", &self.contexts.len())
            .finish()
    }
}

impl Contexts {
    pub fn new() -> Self {
        Self {
            contexts: Vec::new(),
        }
    }

    pub fn contains<T: Any + Send + Sync>(&self) -> bool {
        self.contexts.iter().rev().any(|context| context.is::<T>())
    }

    pub fn push(&mut self, context: impl Any + Send + Sync + 'static) {
        self.contexts.push(Arc::new(context));
    }

    pub fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        for context in self.contexts.iter().rev() {
            if let Some(context) = context.as_ref().downcast_ref::<T>() {
                return Some(context.clone());
            }
        }

        None
    }

    pub fn get_arc<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        for context in self.contexts.iter().rev() {
            if !context.as_ref().is::<T>() {
                continue;
            }

            if let Ok(context) = Arc::downcast::<T>(context.clone()) {
                return Some(context);
            }
        }

        None
    }
}
