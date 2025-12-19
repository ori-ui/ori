use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
};

/// Handle for aborting execution of [`Abortable`].
#[derive(Clone, Debug)]
pub struct Aborter {
    polling: Arc<AtomicBool>,
}

impl Aborter {
    /// Abort the associated [`Abortable`].
    pub fn abort(&self) {
        self.polling.store(false, Ordering::Relaxed);
    }
}

/// [`Future`] that can have its execution aborted.
#[derive(Debug)]
pub struct Abortable<F> {
    future:  F,
    polling: Arc<AtomicBool>,
}

impl<F> Abortable<F> {
    /// Create [`Abortable`] and its [`Aborter`].
    pub fn new(future: F) -> (Abortable<F>, Aborter) {
        let polling = Arc::new(AtomicBool::new(true));

        let aborter = Aborter {
            polling: polling.clone(),
        };

        let abortable = Abortable { future, polling };

        (abortable, aborter)
    }
}

impl<F> Future for Abortable<F>
where
    F: Future<Output = ()>,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.polling.load(Ordering::Relaxed) {
            return Poll::Ready(());
        }

        unsafe {
            let this = self.get_unchecked_mut();
            let future = Pin::new_unchecked(&mut this.future);
            future.poll(cx)
        }
    }
}
