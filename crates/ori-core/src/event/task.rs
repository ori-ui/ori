use std::{
    cell::UnsafeCell,
    future::Future,
    mem::{self, ManuallyDrop},
    pin::Pin,
    sync::Arc,
    task::{Context, RawWaker, RawWakerVTable, Waker},
};

use crate::EventSink;

type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

struct TaskInner {
    future: UnsafeCell<Option<BoxFuture>>,
    event_sink: EventSink,
}

/// A handle to a task.
///
/// When a task is awoken, it will be sent to the event loop, through the
/// provided [`EventSink`] to be polled again. Is is therefore the job of the
/// application shell to ensure that the task is polled.
pub struct Task(Arc<TaskInner>);

impl Task {
    pub fn spawn(event_sink: EventSink, future: impl Future<Output = ()> + Send + 'static) {
        tracing::trace!("spawning task");

        let task = Self(Arc::new(TaskInner {
            future: UnsafeCell::new(Some(Box::pin(future))),
            event_sink,
        }));

        Self::poll_inner(task);
    }

    pub fn poll(&self) {
        tracing::trace!("polling task");
        let task = Self(self.0.clone());
        Self::poll_inner(task);
    }

    fn poll_inner(self) {
        let inner = self.0;

        // SAFETY: This is safe because only one thread can poll a task at a time.
        let future_slot = unsafe { &mut *inner.future.get() };
        if let Some(mut future) = future_slot.take() {
            let waker = inner.waker();
            let context = &mut Context::from_waker(&waker);

            if future.as_mut().poll(context).is_pending() {
                *future_slot = Some(future);
            }
        }
    }
}

impl TaskInner {
    const RAW_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
        Self::clone_arc_raw,
        Self::wake_arc_raw,
        Self::wake_by_ref_arc_raw,
        Self::drop_arc_raw,
    );

    unsafe fn increment_ref_count(ptr: *const ()) {
        let arc = ManuallyDrop::new(Arc::from_raw(ptr as *const Self));
        mem::forget(arc.clone());
    }

    unsafe fn clone_arc_raw(ptr: *const ()) -> RawWaker {
        Self::increment_ref_count(ptr);
        RawWaker::new(ptr, &Self::RAW_WAKER_VTABLE)
    }

    unsafe fn wake_arc_raw(ptr: *const ()) {
        let arc = Arc::from_raw(ptr as *const Self);
        arc.event_sink.emit(Task(arc.clone()));
    }

    unsafe fn wake_by_ref_arc_raw(ptr: *const ()) {
        Self::increment_ref_count(ptr);
        Self::wake_arc_raw(ptr);
    }

    unsafe fn drop_arc_raw(ptr: *const ()) {
        Arc::from_raw(ptr as *const Self);
    }

    fn raw_waker(self: &Arc<Self>) -> RawWaker {
        let ptr = Arc::into_raw(self.clone());
        RawWaker::new(ptr as *const (), &Self::RAW_WAKER_VTABLE)
    }

    fn waker(self: &Arc<Self>) -> Waker {
        unsafe { Waker::from_raw(self.raw_waker()) }
    }
}
