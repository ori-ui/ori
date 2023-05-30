use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Wake, Waker},
};

use parking_lot::Mutex;

use crate::EventSink;

/// A handle to a task.
///
/// When a task is awoken, it will be sent to the event loop, through the
/// provided [`EventSink`] to be polled again. Is is therefore the job of the
/// application shell to ensure that the task is polled.
pub struct Task(Arc<TaskInner>);

impl Task {
    /// Spawns a `future` on the event loop. The future will be polled once and
    /// the waker will send the task to the event loop when it is awoken.
    pub fn spawn(event_sink: EventSink, future: impl Future<Output = ()> + Send + 'static) {
        tracing::trace!("spawning task");

        let task_inner = Arc::new(TaskInner {
            future: Mutex::new(Some(Box::pin(future))),
            event_sink,
        });

        Self::poll_inner(task_inner);
    }

    /// Polls the task.
    pub fn poll(&self) {
        tracing::trace!("polling task");

        Self::poll_inner(self.0.clone());
    }

    fn poll_inner(inner: Arc<TaskInner>) {
        // SAFETY: This is safe because only one thread can poll a task at a time.
        let future_slot = &mut *inner.future.lock();
        if let Some(mut future) = future_slot.take() {
            let waker = Waker::from(inner.clone());
            let context = &mut Context::from_waker(&waker);

            if future.as_mut().poll(context).is_pending() {
                *future_slot = Some(future);
            }
        }
    }
}

type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

struct TaskInner {
    future: Mutex<Option<BoxFuture>>,
    event_sink: EventSink,
}

impl Wake for TaskInner {
    fn wake(self: Arc<Self>) {
        self.event_sink.emit(Task(self.clone()));
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.event_sink.emit(Task(self.clone()));
    }
}

// SAFETY: This is safe because only one thread can poll a task at a time.
unsafe impl Sync for TaskInner {}
