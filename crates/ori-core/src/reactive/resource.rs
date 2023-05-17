use std::{fmt::Debug, marker::PhantomData};

use crate::{ResourceId, Runtime, ScopeId};

pub struct Resource<T: 'static> {
    id: ResourceId,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Clone for Resource<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for Resource<T> {}

impl<T: Clone + Debug> Debug for Resource<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resource")
            .field("id", &self.id)
            .field("data", &self.get())
            .finish()
    }
}

impl<T: Clone + PartialEq> PartialEq for Resource<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T: Clone + Eq> Eq for Resource<T> {}

impl<T> Resource<T> {
    /// Creates a new resource that must be manually disposed.
    pub fn new_leaking(data: T) -> Self {
        Runtime::with_global_runtime(|runtime| Self {
            id: runtime.create_resource(data),
            _marker: PhantomData,
        })
    }

    /// Gets the resource id.
    pub fn id(self) -> ResourceId {
        self.id
    }

    /// # Safety
    /// -
    pub unsafe fn set_id(&mut self, id: ResourceId) {
        self.id = id;
    }

    /// Adds a reference to the resource.
    pub fn reference(self) {
        Runtime::with_global_runtime(|runtime| runtime.reference_resource(self.id));
    }

    pub fn manage(self, scope: ScopeId) -> Self {
        Runtime::with_global_runtime(|runtime| {
            runtime.manage_resource(scope, self.id);
            self
        })
    }

    pub fn get(self) -> Option<T>
    where
        T: Clone,
    {
        Runtime::with_global_runtime(|runtime| {
            // SAFETY: The resource was inserted with the same type as the one we're trying to get.
            unsafe { runtime.get_resource(self.id) }
        })
    }

    pub fn set(self, data: T) -> Result<(), T> {
        Runtime::with_global_runtime(|runtime| {
            // SAFETY: The resource was inserted with the same type as the one we're trying to set.
            unsafe { runtime.set_resource(self.id, data) }
        })
    }

    pub fn take(self) -> Option<T> {
        Runtime::with_global_runtime(|runtime| {
            // SAFETY: The resource was inserted with the same type as the one we're trying to take.
            unsafe { runtime.remove_resource(self.id) }
        })
    }

    pub fn dispose(self) {
        Runtime::with_global_runtime(|runtime| runtime.dispose_resource(self.id));
    }
}
