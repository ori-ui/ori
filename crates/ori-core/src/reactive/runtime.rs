use std::{
    any::Any,
    fmt::Debug,
    mem,
    panic::Location,
    sync::atomic::{AtomicU64, Ordering},
};

use nohash_hasher::{IntMap, IsEnabled};

use crate::{Guard, Lock, Lockable, Sendable};

#[derive(Debug)]
struct RuntimeScope {
    parent: Option<ScopeId>,
    children: Vec<ScopeId>,
    resources: Vec<ResourceId>,
}

struct RuntimeResource {
    #[allow(dead_code)]
    creator: &'static Location<'static>,
    #[cfg(feature = "multi-thread")]
    data: Box<dyn Any + Send>,
    #[cfg(not(feature = "multi-thread"))]
    data: Box<dyn Any>,
    refs: u32,
}

#[derive(Default)]
pub struct Runtime {
    is_global: bool,
    scopes: Lock<IntMap<ScopeId, RuntimeScope>>,
    resources: Lock<IntMap<ResourceId, RuntimeResource>>,
}

impl Runtime {
    fn global() -> Self {
        Self {
            is_global: true,
            scopes: Lock::new(IntMap::default()),
            resources: Lock::new(IntMap::default()),
        }
    }

    #[cfg(feature = "multi-thread")]
    pub fn with_global_runtime<T>(f: impl FnOnce(&Runtime) -> T) -> T {
        lazy_static::lazy_static! {
            static ref RUNTIME: Runtime = Runtime::global();
        }

        f(&RUNTIME)
    }

    #[cfg(not(feature = "multi-thread"))]
    pub fn with_global_runtime<T>(f: impl FnOnce(&Runtime) -> T) -> T {
        std::thread_local! {
            static RUNTIME: Runtime = Runtime::global();
        }

        RUNTIME.with(f)
    }

    fn scopes(&self) -> Guard<'_, IntMap<ScopeId, RuntimeScope>> {
        self.scopes.lock_mut()
    }

    fn resources(&self) -> Guard<'_, IntMap<ResourceId, RuntimeResource>> {
        self.resources.lock_mut()
    }

    pub fn create_scope(&self, parent: Option<ScopeId>) -> ScopeId {
        let id = ScopeId::new();
        let mut scopes = self.scopes();
        scopes.insert(
            id,
            RuntimeScope {
                parent,
                children: Vec::new(),
                resources: Vec::new(),
            },
        );

        if let Some(parent) = parent {
            if let Some(parent) = scopes.get_mut(&parent) {
                parent.children.push(id);
            }
        }

        id
    }

    pub fn scope_parent(&self, scope: ScopeId) -> Option<ScopeId> {
        let scopes = self.scopes();
        let scope = scopes.get(&scope)?;
        scope.parent
    }

    pub fn manage_resource(&self, scope: ScopeId, resource: ResourceId) {
        tracing::trace!("managing resource {:?} in scope {:?}", resource, scope);

        let mut scopes = self.scopes();
        let scope = scopes.get_mut(&scope).unwrap();
        scope.resources.push(resource);
    }

    pub fn dispose_scope(&self, scope: ScopeId) {
        let scope = {
            let mut scopes = self.scopes();
            scopes.remove(&scope).unwrap()
        };

        tracing::trace!("disposing scope {:?}", scope);

        for child in scope.children {
            self.dispose_scope(child);
        }

        for resource in scope.resources {
            self.dispose_resource(resource);
        }
    }

    #[track_caller]
    pub fn create_resource<T: Sendable + 'static>(&self, data: T) -> ResourceId {
        let id = ResourceId::new();

        tracing::trace!("creating resource {:?} at {}", id, Location::caller());

        self.resources().insert(
            id,
            RuntimeResource {
                creator: Location::caller(),
                data: Box::new(data),
                refs: 0,
            },
        );

        id
    }

    /// Adds a reference to the resource at `id`.
    pub fn reference_resource(&self, id: ResourceId) {
        tracing::trace!("referencing resource {:?}", id);

        let mut resources = self.resources();
        if let Some(resource) = resources.get_mut(&id) {
            resource.refs += 1;
        }
    }

    /// # Safety
    /// - The caller must ensure that the resource stored at `id` is of type `T`.
    pub unsafe fn get_resource<T: Clone + 'static>(&self, id: ResourceId) -> Option<T> {
        let resources = self.resources();
        let resource = resources.get(&id)?;

        let ptr = resource.data.as_ref() as *const _ as *const T;
        Some(unsafe { &*ptr }.clone())
    }

    /// Sets the resource at `id` to `value`. This ignores the reference count.
    ///
    /// # Safety
    /// - The caller must ensure that the resource stored at `id` is of type `T`.
    #[track_caller]
    pub unsafe fn set_resource<T: Sendable + 'static>(
        &self,
        id: ResourceId,
        value: T,
    ) -> Result<(), T> {
        tracing::trace!("setting resource {:?} at {}", id, Location::caller());

        let old = match self.resources().get_mut(&id) {
            Some(resource) => mem::replace(&mut resource.data, Box::new(value)),
            None => return Err(value),
        };

        drop(old);

        Ok(())
    }

    /// Takes the resource out of the runtime, returning it. This ignores the reference count.
    ///
    /// # Safety
    /// - The caller must ensure that the resource stored at `id` is of type `T`.
    #[track_caller]
    pub unsafe fn remove_resource<T: 'static>(&self, id: ResourceId) -> Option<T> {
        tracing::trace!("removing resource {:?}, at {}", id, Location::caller());

        let resource = {
            let mut resources = self.resources();
            resources.remove(&id)?
        };

        let ptr = Box::into_raw(resource.data) as *mut T;
        Some(unsafe { *Box::from_raw(ptr) })
    }

    /// Disposes a resource, decrementing its reference count.
    /// If the reference count reaches zero, the resource is removed from the runtime.
    #[track_caller]
    pub fn dispose_resource(&self, id: ResourceId) {
        tracing::trace!("disposing resource {:?} at {}", id, Location::caller());

        let resource = {
            let mut resources = self.resources();

            let Some(resource) = resources.get_mut(&id) else { return };

            if resource.refs > 0 {
                resource.refs -= 1;
                return;
            }

            resources.remove(&id)
        };

        drop(resource);
    }

    pub fn clear(&self) {
        tracing::trace!("clearing runtime");

        let scopes = mem::take(&mut *self.scopes());
        let resources = mem::take(&mut *self.resources());

        drop(scopes);
        drop(resources);
    }

    pub fn forget(&self) {
        tracing::trace!("forgetting runtime");

        let scopes = mem::take(&mut *self.scopes());
        let resources = mem::take(&mut *self.resources());

        mem::forget(scopes);
        mem::forget(resources);
    }

    pub fn stop_global() {
        Self::with_global_runtime(|runtime| {
            runtime.clear();
        });
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        if self.is_global {
            self.forget();
        }
    }
}

macro_rules! define_ids {
    ($($name:ident),* $(,)?) => {$(
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name {
            id: u64,
        }

        impl $name {
            pub fn new() -> Self {
                static NEXT_ID: AtomicU64 = AtomicU64::new(0);

                Self {
                    id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
                }
            }

            pub fn as_u64(self) -> u64 {
                self.id
            }
        }

        impl IsEnabled for $name {}
    )*};
}

define_ids!(ScopeId, ResourceId);
