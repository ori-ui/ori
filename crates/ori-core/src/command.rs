//! A channel for sending commands to the user interface.

use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    fmt::Debug,
    future::Future,
    mem::ManuallyDrop,
    pin::Pin,
    sync::Arc,
    task::{Context, RawWaker, RawWakerVTable, Waker},
};

use crossbeam_channel::{Receiver, Sender};

/// A waker for the event loop.
///
/// When called, the event loop should wake up and process any pending commands,
/// by calling [`Ui::handle_commands()`](crate::ui::Ui::handle_commands).
#[derive(Clone)]
pub struct CommandWaker(Arc<dyn Fn() + Send + Sync>);

impl CommandWaker {
    /// Create a new [`CommandWaker`].
    pub fn new(waker: impl Fn() + Send + Sync + 'static) -> Self {
        Self(Arc::new(waker))
    }

    /// Wake the event loop.
    pub fn wake(&self) {
        (self.0)();
    }
}

impl From<Arc<dyn Fn() + Send + Sync>> for CommandWaker {
    fn from(waker: Arc<dyn Fn() + Send + Sync>) -> Self {
        Self(waker)
    }
}

impl Debug for CommandWaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandWaker").finish()
    }
}

/// A command.
#[derive(Debug)]
pub struct Command {
    type_id: TypeId,
    data: Box<dyn Any + Send>,
    name: &'static str,
}

impl Command {
    /// Create a new command.
    pub fn new<T: Any + Send>(command: T) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            data: Box::new(command),
            name: std::any::type_name::<T>(),
        }
    }

    /// Check whether the command is of type `T`.
    pub fn is<T: Any>(&self) -> bool {
        self.type_id == TypeId::of::<T>()
    }

    /// Try to downcast the command to `T`.
    pub fn get<T: Any>(&self) -> Option<&T> {
        if self.is::<T>() {
            // SAFETY: We just checked that the type is correct.
            //
            // We need unsafe here because <dyn Any>::downcast_ref does a dynamic call to
            // check the type, which is slow... This function is called a lot, so we want
            // to avoid that.
            unsafe { Some(&*(self.data.as_ref() as *const _ as *const T)) }
        } else {
            None
        }
    }

    /// Get the name of the command.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Convert the command into a boxed [`Any`] value.
    pub fn to_any(self) -> Box<dyn Any + Send> {
        self.data
    }
}

/// A clonable channel for sending [`Command`]s.
#[derive(Clone)]
pub struct CommandProxy {
    tx: Sender<Command>,
    waker: CommandWaker,
}

impl CommandProxy {
    /// Create a new [`CommandProxy`] channel.
    pub fn new(waker: CommandWaker) -> (Self, CommandReceiver) {
        let (tx, rx) = crossbeam_channel::unbounded();
        (Self { tx, waker }, CommandReceiver { rx })
    }

    /// Wake the event loop.
    pub fn wake(&self) {
        self.waker.wake();
    }

    /// Send a command without waking the event loop.
    ///
    /// This is almost never what you want to do. Use [`CommandProxy::cmd`] instead.
    pub fn cmd_silent(&self, command: Command) {
        if let Err(err) = self.tx.send(command) {
            tracing::warn!("failed to send command: {}", err);
        }
    }

    /// Send a command.
    pub fn cmd(&self, command: impl Any + Send) {
        self.cmd_silent(Command::new(command));
        self.wake();
    }

    /// Spawn a future that is polled when commands are handled.
    pub fn spawn_async(&self, future: impl Future<Output = ()> + Send + 'static) {
        let task = Arc::new(CommandTask::new(self, future));

        // SAFETY: the task was just created, so it's impossible for there to be any clones of the
        // Arc, which means we have unique access to the task.
        unsafe { task.poll() };
    }

    /// Spawn a future sending a command when it completes.
    ///
    /// See [`CommandProxy::spawn_async`] for more information.
    pub fn cmd_async<T: Any + Send>(&self, future: impl Future<Output = T> + Send + 'static) {
        let proxy = self.clone();

        self.spawn_async(async move {
            proxy.cmd(future.await);
        });
    }
}

impl Debug for CommandProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandProxy").finish()
    }
}

/// A receiver for [`Command`]s.
#[derive(Debug)]
pub struct CommandReceiver {
    rx: Receiver<Command>,
}

impl CommandReceiver {
    fn try_recv_inner(&self) -> Option<Command> {
        self.rx.try_recv().ok()
    }

    /// Try receive a command.
    pub fn try_recv(&self) -> Option<Command> {
        let mut command = self.try_recv_inner()?;

        while let Some(task) = command.get::<Arc<CommandTask>>() {
            // SAFETY: the only way to send a CommandTask is through CommandProxy::spawn_async,
            // since CommandTask is not public, and CommandReceiver does not implement Clone or
            // Sync. therefore, it is impossible for `task` to be polled from multiple threads at
            // once.
            unsafe { task.poll() };
            command = self.try_recv_inner()?;
        }

        Some(command)
    }
}

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

struct CommandTask {
    proxy: CommandProxy,
    future: UnsafeCell<Option<BoxFuture<'static, ()>>>,
}

// SAFETY: CommandTask::future is only ever accessed from one thread at a time.
unsafe impl Sync for CommandTask {}

impl CommandTask {
    fn new(proxy: &CommandProxy, future: impl Future<Output = ()> + Send + 'static) -> Self {
        Self {
            proxy: proxy.clone(),
            future: UnsafeCell::new(Some(Box::pin(future))),
        }
    }

    fn raw_waker_vtable() -> &'static RawWakerVTable {
        &RawWakerVTable::new(
            CommandTask::waker_clone,
            CommandTask::waker_wake,
            CommandTask::waker_wake_by_ref,
            CommandTask::waker_drop,
        )
    }

    unsafe fn increase_refcount(data: *const ()) {
        let arc = ManuallyDrop::new(Arc::from_raw(data.cast::<Self>()));
        let _arc_clone = arc.clone();
    }

    unsafe fn waker_clone(data: *const ()) -> RawWaker {
        Self::increase_refcount(data);
        RawWaker::new(data, Self::raw_waker_vtable())
    }

    unsafe fn waker_wake(data: *const ()) {
        let arc = Arc::from_raw(data.cast::<Self>());
        arc.proxy.cmd(arc.clone());
    }

    unsafe fn waker_wake_by_ref(data: *const ()) {
        let arc = ManuallyDrop::new(Arc::from_raw(data.cast::<Self>()));
        let task: Arc<Self> = (*arc).clone();
        arc.proxy.cmd(task);
    }

    unsafe fn waker_drop(data: *const ()) {
        drop(Arc::from_raw(data.cast::<Self>()));
    }

    fn raw_waker(self: &Arc<Self>) -> RawWaker {
        let data = Arc::into_raw(self.clone());
        RawWaker::new(data.cast(), Self::raw_waker_vtable())
    }

    // SAFETY: must never be called anywhere other than `CommandReceiver::try_recv`
    unsafe fn poll(self: &Arc<Self>) {
        let future_slot = &mut *self.future.get();

        if let Some(mut future) = future_slot.take() {
            let waker = Waker::from_raw(self.raw_waker());
            let mut cx = Context::from_waker(&waker);

            if future.as_mut().poll(&mut cx).is_pending() {
                *future_slot = Some(future);
            }
        }
    }
}
