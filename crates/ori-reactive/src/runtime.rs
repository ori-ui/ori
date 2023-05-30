use std::{
    any::Any,
    fmt::Debug,
    mem,
    panic::Location,
    sync::atomic::{AtomicUsize, Ordering},
};

use sharded::Map;

#[derive(Debug)]
struct RuntimeScope {
    parent: Option<ScopeId>,
    children: Vec<ScopeId>,
    resources: Vec<ResourceId>,
}

struct RuntimeResource {
    #[allow(dead_code)]
    creator: &'static Location<'static>,
    references: u32,
    data: Box<dyn Any + Send + Sync>,
}

impl Debug for RuntimeResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeResource")
            .field("creator", &self.creator)
            .field("refs", &self.references)
            .finish()
    }
}

/// A runtime that manages scopes and resources.
///
/// Scopes are used to manage the lifetime of resources. When a scope is disposed, all resources
/// that were created in that scope are disposed as well.
///
/// Resources are created with [`Runtime::create_resource`]. They are reference counted, and
/// disposed when their reference count reaches zero.
#[derive(Default)]
pub struct Runtime {
    scopes: Map<ScopeId, RuntimeScope>,
    resources: Map<ResourceId, RuntimeResource>,
}

impl Runtime {
    fn new_global() -> Self {
        Self {
            scopes: Map::default(),
            resources: Map::default(),
        }
    }

    /// Returns a reference to the global runtime.
    pub fn global() -> &'static Self {
        lazy_static::lazy_static! {
            static ref RUNTIME: Runtime = Runtime::new_global();
        }

        &RUNTIME
    }

    /// Creates a new scope.
    pub fn create_scope(&self, parent: Option<ScopeId>) -> ScopeId {
        let id = ScopeId::new();

        self.scopes.insert(
            id,
            RuntimeScope {
                parent,
                children: Vec::new(),
                resources: Vec::new(),
            },
        );

        if let Some(parent) = parent {
            let (key, mut shard) = self.scopes.write(parent);

            if let Some(parent) = shard.get_mut(key) {
                parent.children.push(id);
            }
        }

        id
    }

    /// Returns the parent of the scope at `scope`.
    pub fn scope_parent(&self, scope: ScopeId) -> Option<ScopeId> {
        let (key, shard) = self.scopes.read(&scope);
        shard.get(key)?.parent
    }

    /// Manages the resource at `resource` in the scope at `scope`.
    #[track_caller]
    pub fn manage_resource(&self, scope: ScopeId, resource: ResourceId) {
        tracing::trace!("managing resource {:?} in scope {:?}", resource, scope);

        let (key, mut shard) = self.scopes.write(scope);
        if let Some(scope) = shard.get_mut(key) {
            scope.resources.push(resource);
        }
    }

    /// Disposes the scope at `scope`.
    #[track_caller]
    pub fn dispose_scope(&self, scope: ScopeId) {
        let scope = {
            match self.scopes.remove(scope) {
                Some(scope) => scope,
                None => return,
            }
        };

        tracing::trace!("disposing scope {:?}", scope);

        for child in scope.children {
            self.dispose_scope(child);
        }

        for resource in scope.resources {
            self.dispose_resource(resource);
        }
    }

    /// Creates a new resource with the given `value`.
    ///
    /// Resources are reference counted, and are disposed when their reference count reaches zero.
    #[track_caller]
    pub fn create_resource<T: Send + Sync + 'static>(&self, value: T) -> ResourceId {
        let id = ResourceId::new();

        tracing::trace!("creating resource {:?} at {}", id, Location::caller());

        let resource = RuntimeResource {
            creator: Location::caller(),
            data: Box::new(value),
            references: 0,
        };

        self.resources.insert(id, resource);

        id
    }

    /// Adds a reference to the resource at `id`.
    #[track_caller]
    pub fn reference_resource(&self, id: ResourceId) {
        tracing::trace!("referencing resource {:?}", id);

        let (key, mut shard) = self.resources.write(id);
        if let Some(resource) = shard.get_mut(key) {
            resource.references += 1;
        }
    }

    /// Gets the reference count of the resource at `id`.
    #[track_caller]
    pub fn get_reference_count(&self, id: ResourceId) -> Option<u32> {
        let (key, shard) = self.resources.read(&id);
        shard.get(key).map(|r| r.references + 1)
    }

    /// Gets and clone of the value of the resource at `id`.
    ///
    /// **Note** that if `T::clone` accesses the runtime, a deadlock is likely to occur.
    ///
    /// # Safety
    /// - The caller must ensure that the resource stored at `id` is of type `T`.
    #[track_caller]
    pub unsafe fn get_resource<T: Clone + 'static>(&self, id: ResourceId) -> Option<T> {
        self.with_resource(id, T::clone)
    }

    /// Runs `f` with a reference to the resource at `id`.
    ///
    /// **Note** that accessing the runtime from within `f` should be avoided at all costs, as it
    /// is likely to cause a deadlock.
    ///
    /// # Safety
    /// - The caller must ensure that the resource stored at `id` is of type `T`.
    #[track_caller]
    pub unsafe fn with_resource<T: 'static, U>(
        &self,
        id: ResourceId,
        f: impl FnOnce(&T) -> U,
    ) -> Option<U> {
        tracing::trace!("getting resource {:?} at {}", id, Location::caller());

        let (key, shard) = self.resources.read(&id);
        let resource = shard.get(key)?;

        let ptr = resource.data.as_ref() as *const _ as *const T;
        Some(f(&*ptr))
    }

    /// Runs `f` with a mutable reference to the resource at `id`.
    ///
    /// **Note** that accessing the runtime from within `f` should be avoided at all costs, as it
    /// is likely to cause a deadlock.
    ///
    /// # Safety
    /// - The caller must ensure that the resource stored at `id` is of type `T`.
    #[track_caller]
    pub unsafe fn with_resource_mut<T: 'static, U>(
        &self,
        id: ResourceId,
        f: impl FnOnce(&mut T) -> U,
    ) -> Option<U> {
        tracing::trace!("getting resource {:?} at {}", id, Location::caller());

        let (key, mut shard) = self.resources.write(id);
        let resource = shard.get_mut(key)?;

        let ptr = resource.data.as_mut() as *mut _ as *mut T;
        Some(f(unsafe { &mut *ptr }))
    }

    /// Sets the resource at `id` to `value`. This ignores the reference count.
    ///
    /// # Safety
    /// - The caller must ensure that the resource stored at `id` is of type `T`.
    #[track_caller]
    pub unsafe fn set_resource<T: Send + Sync + 'static>(
        &self,
        id: ResourceId,
        value: T,
    ) -> Result<(), T> {
        tracing::trace!("setting resource {:?} at {}", id, Location::caller());

        let (key, mut shard) = self.resources.write(id);
        let old = match shard.get_mut(key) {
            Some(resource) => mem::replace(&mut resource.data, Box::new(value)),
            None => return Err(value),
        };

        drop(old);

        Ok(())
    }

    /// Takes the resource out of the runtime, returning it.
    ///
    /// **Note** that this ignores the reference count, and should therefore be used with caution.
    ///
    /// # Safety
    /// - The caller must ensure that the resource stored at `id` is of type `T`.
    #[track_caller]
    pub unsafe fn remove_resource<T: 'static>(&self, id: ResourceId) -> Option<T> {
        tracing::trace!("removing resource {:?}, at {}", id, Location::caller());

        let resource = self.resources.remove(id)?;

        let ptr = Box::into_raw(resource.data) as *mut T;
        Some(unsafe { *Box::from_raw(ptr) })
    }

    /// Disposes a resource, decrementing its reference count.
    /// If the reference count reaches zero, the resource is removed from the runtime.
    #[track_caller]
    pub fn dispose_resource(&self, id: ResourceId) {
        tracing::trace!("disposing resource {:?} at {}", id, Location::caller());

        let resource = {
            let (key, mut shard) = self.resources.write(id);
            let Some(resource) = shard.get_mut(key) else { return };

            if resource.references > 0 {
                resource.references -= 1;
                return;
            }

            drop(shard);

            self.resources.remove(id)
        };

        drop(resource);
    }
}

macro_rules! define_ids {
    ($($name:ident),* $(,)?) => {$(
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name {
            id: usize,
        }

        impl $name {
            #[inline(always)]
            pub fn new() -> Self {
                static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

                Self {
                    id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
                }
            }

            pub const fn as_usize(self) -> usize {
                self.id
            }
        }
    )*};
}

define_ids!(ScopeId, ResourceId);
