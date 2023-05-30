use std::{fmt::Debug, marker::PhantomData};

use crate::{ResourceId, Runtime, ScopeId};

/// A copyable handle to a resource, managed by the global [`Runtime`].
#[repr(transparent)]
pub struct Resource<T: 'static> {
    id: ResourceId,
    marker: PhantomData<fn() -> T>,
}

impl<T> Clone for Resource<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            marker: PhantomData,
        }
    }
}

impl<T> Copy for Resource<T> {}

impl<T: Send + Sync + Clone + Debug> Debug for Resource<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resource")
            .field("id", &self.id)
            .field("data", &self.get())
            .finish()
    }
}

impl<T: Send + Sync + Clone + PartialEq> PartialEq for Resource<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T: Send + Sync + Clone + Eq> Eq for Resource<T> {}

impl<T: Send + Sync> Resource<T> {
    /// Creates a new resource that must be manually disposed, see [`Resource::dispose`].
    ///
    /// If the resource is not disposed, it will leak.
    #[track_caller]
    pub fn new_leaking(data: T) -> Self {
        Self {
            id: Runtime::global().create_resource(data),
            marker: PhantomData,
        }
    }

    /// Gets a clone of the resource, see [`Runtime::get_resource`].
    #[track_caller]
    pub fn get(self) -> Option<T>
    where
        T: Clone,
    {
        // SAFETY: The resource was inserted with the same type as the one we're trying to get.
        unsafe { Runtime::global().get_resource(self.id) }
    }

    /// Runs `f` with a reference to the resource, see [`Runtime::with_resource`].
    #[track_caller]
    pub fn with<U>(self, f: impl FnOnce(&T) -> U) -> Option<U> {
        // SAFETY: The resource was inserted with the same type as the one we're trying to get.
        unsafe { Runtime::global().with_resource(self.id, f) }
    }

    /// Runs `f` with a mutable reference to the resource, see [`Runtime::with_resource_mut`].
    #[track_caller]
    pub fn with_mut<U>(self, f: impl FnOnce(&mut T) -> U) -> Option<U> {
        // SAFETY: The resource was inserted with the same type as the one we're trying to get.
        unsafe { Runtime::global().with_resource_mut(self.id, f) }
    }

    /// Sets the resource, see [`Runtime::set_resource`].
    #[track_caller]
    pub fn set(self, data: T) -> Result<(), T> {
        // SAFETY: The resource was inserted with the same type as the one we're trying to set.
        unsafe { Runtime::global().set_resource(self.id, data) }
    }

    /// Takes the resource, see [`Runtime::remove_resource`].
    #[track_caller]
    pub fn take(self) -> Option<T> {
        // SAFETY: The resource was inserted with the same type as the one we're trying to take.
        unsafe { Runtime::global().remove_resource(self.id) }
    }
}

impl<T> Resource<T> {
    /// Gets the resource id.
    pub fn id(self) -> ResourceId {
        self.id
    }

    /// Sets the internal resource id.
    ///
    /// # Safety
    /// - The resource at `id` must be of type `T`.
    pub unsafe fn set_id(&mut self, id: ResourceId) {
        self.id = id;
    }

    /// Adds a reference to the resource, see [`Runtime::reference_resource`].
    #[track_caller]
    pub fn reference(self) {
        Runtime::global().reference_resource(self.id);
    }

    /// Registers the to be managed by the scope at `scope`, see [`Runtime::manage_resource`].
    #[track_caller]
    pub fn manage(self, scope: ScopeId) {
        Runtime::global().manage_resource(scope, self.id);
    }

    /// Disposes the resource, see [`Runtime::dispose_resource`].
    #[track_caller]
    pub fn dispose(self) {
        Runtime::global().dispose_resource(self.id);
    }
}
