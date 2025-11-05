use std::{
    any::Any,
    sync::atomic::{AtomicU64, Ordering},
};

pub struct Event {
    target: Option<ViewId>,
    item: Option<Box<dyn Any + Send + Sync>>,
}

impl Event {
    pub fn new(
        item: impl Any + Send + Sync,
        target: impl Into<Option<ViewId>>,
    ) -> Self {
        Self {
            target: target.into(),
            item: Some(Box::new(item)),
        }
    }

    pub fn target(&self) -> Option<ViewId> {
        self.target
    }

    pub fn is_target(&self, id: ViewId) -> bool {
        self.target() == Some(id)
    }

    pub fn is<T: Any + Send + Sync>(&self) -> bool {
        self.item.as_ref().is_some_and(|item| item.is::<T>())
    }

    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.item.as_ref().and_then(|item| item.downcast_ref())
    }

    pub fn get_mut<T: Any + Send + Sync>(&mut self) -> Option<&mut T> {
        self.item.as_mut().and_then(|item| item.downcast_mut())
    }

    pub fn take<T: Any + Send + Sync>(&mut self) -> Option<T> {
        match self.item.take()?.downcast() {
            Ok(item) => Some(*item),
            Err(item) => {
                self.item = Some(item);
                None
            }
        }
    }

    pub fn get_targeted<T: Any + Send + Sync>(&self, id: ViewId) -> Option<&T> {
        self.get().filter(|_| self.is_target(id))
    }

    pub fn take_targeted<T: Any + Send + Sync>(
        &mut self,
        id: ViewId,
    ) -> Option<T> {
        self.is_target(id).then(|| self.take()).flatten()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ViewId {
    id: u64,
}

impl ViewId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        Self::from_u64(NEXT_ID.fetch_add(1, Ordering::SeqCst))
    }

    pub const fn from_u64(id: u64) -> Self {
        Self { id }
    }

    pub const fn as_u64(self) -> u64 {
        self.id
    }
}
