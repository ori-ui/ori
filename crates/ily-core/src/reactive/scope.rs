use std::{
    cell::{Cell, RefCell},
    iter::FilterMap,
    marker::PhantomData,
    mem,
    ops::Index,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
    vec::IntoIter,
};

use crate::{ReadSignal, SharedSignal, Signal};

trait Item {}
impl<T> Item for T {}

#[derive(Default, Debug)]
struct ScopeArena<'a> {
    items: RefCell<Vec<*mut (dyn Item + 'a)>>,
}

impl<'a> ScopeArena<'a> {
    pub fn alloc_static<T: 'static>(&self, item: T) -> &'a mut T {
        let item = Box::into_raw(Box::new(item));
        self.items.borrow_mut().push(item);
        unsafe { &mut *item }
    }

    /// # Safety
    /// - `item` must never reference any other item in the arena in it's [`Drop`] implementation.
    pub unsafe fn alloc<T: 'a>(&self, item: T) -> &'a mut T {
        let item = Box::into_raw(Box::new(item));
        self.items.borrow_mut().push(item);
        &mut *item
    }

    /// Disposes all items in the arena.
    ///
    /// Calling this multiple times is a no-op.
    ///
    /// # Safety
    /// - There must be no other references to any item in the arena.
    pub unsafe fn dispose(&self) {
        let mut items = self.items.borrow_mut();
        Self::dispose_inner(&mut items);
    }

    unsafe fn dispose_inner(items: &mut Vec<*mut (dyn Item + 'a)>) {
        for &item in items.iter().rev() {
            // SAFETY: `item` is the only reference to the boxed value, so it's safe to drop it.
            unsafe { Box::from_raw(item) };
        }
        items.clear();
    }
}

impl<'a> Drop for ScopeArena<'a> {
    fn drop(&mut self) {
        let items = self.items.get_mut();
        unsafe { Self::dispose_inner(items) };
    }
}

#[derive(Clone, Debug)]
struct Sparse<T> {
    items: Vec<Option<T>>,
    free: Vec<usize>,
}

impl<T> Sparse<T> {
    pub const fn new() -> Self {
        Self {
            items: Vec::new(),
            free: Vec::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)?.as_ref()
    }

    pub fn insert(&mut self, item: T) -> usize {
        if let Some(index) = self.free.pop() {
            self.items[index] = Some(item);
            index
        } else {
            let index = self.items.len();
            self.items.push(Some(item));
            index
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        let item = self.items[index].take();
        self.free.push(index);
        item
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter().filter_map(|item| item.as_ref())
    }
}

impl<T> Default for Sparse<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Index<usize> for Sparse<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.items[index].as_ref().unwrap()
    }
}

impl<T> IntoIterator for Sparse<T> {
    type Item = T;
    type IntoIter = FilterMap<IntoIter<Option<T>>, fn(Option<T>) -> Option<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter().filter_map(|item| item)
    }
}

#[derive(Debug)]
struct RawScope<'a> {
    /// The arena that holds all items in this scope.
    arena: ScopeArena<'a>,

    /// A reference to the parent scope.
    #[allow(dead_code)]
    parent: Option<&'a RawScope<'a>>,

    /// A lock that prevents the scope from being dropped while an effect is running.
    /// This is used to prevent a use-after-free.
    drop_lock: AtomicUsize,

    /// A list of child scopes.
    children: RefCell<Sparse<*mut RawScope<'a>>>,

    /// A marker that ensures that 'a is invariant
    marker: PhantomData<&'a mut &'a ()>,
}

impl<'a> RawScope<'a> {
    fn new() -> Self {
        Self {
            arena: ScopeArena::default(),
            parent: None,
            drop_lock: AtomicUsize::new(0),
            children: RefCell::new(Sparse::new()),
            marker: PhantomData,
        }
    }

    fn child(parent: &'a RawScope<'a>) -> Self {
        Self {
            arena: ScopeArena::default(),
            parent: Some(parent),
            drop_lock: AtomicUsize::new(0),
            children: RefCell::new(Sparse::new()),
            marker: PhantomData,
        }
    }

    fn push_child(&self, child: *mut RawScope<'a>) -> usize {
        let mut children = self.children.borrow_mut();
        children.insert(child)
    }

    fn is_drop_locked(&self) -> bool {
        if self.drop_lock.load(Ordering::Acquire) > 0 {
            return true;
        }

        self.is_child_scopes_drop_locked()
    }

    fn is_child_scope_drop_locked(&self, index: usize) -> bool {
        let children = self.children.borrow();
        if let Some(&child) = children.get(index) {
            let child = unsafe { &*child };
            child.is_drop_locked()
        } else {
            false
        }
    }

    fn is_child_scopes_drop_locked(&self) -> bool {
        self.children.borrow().iter().any(|&child| {
            let child = unsafe { &*child };
            child.is_drop_locked()
        })
    }

    unsafe fn dispose(&self) {
        let mut children = self.children.borrow_mut();

        for child in mem::take(&mut *children).into_iter() {
            let cx = Box::from_raw(child);
            cx.dispose();
        }

        self.arena.dispose();
    }
}

impl<'a> Drop for RawScope<'a> {
    fn drop(&mut self) {
        unsafe { self.dispose() };
    }
}

pub type Scope<'a> = BoundedScope<'a, 'a>;

#[derive(Clone, Copy, Debug)]
pub struct BoundedScope<'a, 'b: 'a> {
    raw: &'a RawScope<'a>,
    marker: PhantomData<&'b ()>,
}

impl<'a> Scope<'a> {
    /// Creates a new scope.
    ///
    /// This function returns a [`ScopeDisposer`] that must be used to dispose of the scope.
    /// If the disposer is not used, the scope will leak memory.
    #[must_use = "not calling `dispose` will leak memory"]
    pub fn new(f: impl FnOnce(Scope<'a>) + 'a) -> ScopeDisposer<'a> {
        let raw = Box::into_raw(Box::new(RawScope::new()));
        let scope = Scope {
            raw: unsafe { &*raw },
            marker: PhantomData,
        };
        super::effect::untrack(|| f(scope));
        ScopeDisposer::root(raw)
    }

    pub(crate) fn drop_lock(&self) {
        self.raw.drop_lock.fetch_add(1, Ordering::AcqRel);
    }

    pub(crate) fn release_drop_lock(&self) {
        self.raw.drop_lock.fetch_sub(1, Ordering::AcqRel);
    }

    /// Creates a new scope and immediately disposes it.
    pub fn immediate(f: impl FnOnce(Scope<'a>) + 'a) {
        let disposer = Self::new(f);

        // SAFETY: the scope is not accessed after this point.
        unsafe { disposer.dispose() };
    }

    /// Creates a new child scope.
    pub fn child(self, f: impl for<'b> FnOnce(BoundedScope<'b, 'a>)) -> ScopeDisposer<'a> {
        let raw = Box::into_raw(Box::new(RawScope::child(self.raw)));
        let index = self.raw.push_child(raw);
        let scope = Scope {
            raw: unsafe { &*raw },
            marker: PhantomData,
        };
        f(scope);
        ScopeDisposer::child(self.raw, index)
    }

    /// Allocates an item in the scope.
    pub fn alloc<T: 'static>(&self, item: T) -> &'a T {
        self.raw.arena.alloc_static(item)
    }

    /// Allocates an item in the scope.
    ///
    /// # Safety
    /// - `item` must never reference any other item in the arena in it's [`Drop`] implementation.
    pub unsafe fn alloc_unsafe<T: 'a>(self, item: T) -> &'a T {
        self.raw.arena.alloc(item)
    }

    /// Allocates an item in the scope.
    pub fn alloc_mut<T: 'static>(&self, item: T) -> &'a mut T {
        self.raw.arena.alloc_static(item)
    }

    /// Allocates an item in the scope.
    ///
    /// # Safety
    /// - `item` must never reference any other item in the arena in it's [`Drop`] implementation.
    pub unsafe fn alloc_mut_unsafe<T: 'a>(self, item: T) -> &'a mut T {
        self.raw.arena.alloc(item)
    }

    /// Creates a signal in the scope.
    pub fn signal<T: 'static>(self, value: T) -> &'a Signal<T> {
        self.alloc(Signal::new(value))
    }

    /// Runs a scope without tracking any dependencies.
    pub fn untrack<T>(self, f: impl FnOnce() -> T) -> T {
        super::effect::untrack(f)
    }

    /// Creates an effect.
    ///
    /// Effects are callbacks that are run whenever a dependency changes (eg. a signal is updated).
    ///
    /// # Examples
    ///
    /// ```
    /// # use ily_core::*;
    /// # Scope::immediate(|cx| {
    /// let signal = cx.signal(0);
    ///
    /// cx.effect(|| {
    ///     println!("signal is {}", signal.get()); // prints "signal is 0"
    /// });
    ///
    /// signal.set(1); // prints "signal is 1"
    /// # });
    /// ```
    pub fn effect(self, f: impl FnMut() + 'a) {
        super::effect::create_effect(self, f);
    }

    /// Creates an effect in a child scope. See [`Scope::effect`].
    #[track_caller]
    pub fn effect_scoped(self, mut f: impl for<'b> FnMut(BoundedScope<'b, 'a>) + 'a) {
        let mut disposer = None::<ScopeDisposer<'a>>;
        self.effect(move || {
            if let Some(disposer) = disposer.take() {
                if !disposer.is_drop_locked() {
                    // SAFETY: the scope is not accessed after this point.
                    unsafe { disposer.dispose() };
                } else {
                    tracing::trace!("scope is drop locked, leaking disposer");
                }
            }

            disposer = Some(self.child(|cx| {
                f(cx);
            }));
        });
    }

    /// Creates a signal that is recomputed every time a dependency is updated.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ily_core::*;
    /// # Scope::immediate(|cx| {
    /// let signal = cx.signal(0);
    ///
    /// let memo = cx.memo(|| *signal.get() * 2);
    ///
    /// assert_eq!(*memo, 0);
    ///
    /// signal.set(2);
    /// assert_eq!(*memo, 4);
    /// # });
    /// ```
    pub fn memo<T: 'static>(self, mut f: impl FnMut() -> T + 'a) -> &'a ReadSignal<T> {
        let signal = Rc::new(Cell::new(None::<&Signal<T>>));

        self.effect({
            let signal = signal.clone();
            move || {
                let value = f();
                if let Some(signal) = signal.get() {
                    signal.set(value);
                } else {
                    signal.set(Some(self.signal(value)));
                }
            }
        });

        signal.get().unwrap()
    }

    pub fn dynamic<T: 'static>(
        self,
        mut f: impl FnMut(BoundedScope<'_, 'a>) -> T + 'a,
    ) -> SharedSignal<T> {
        let signal = self.alloc(RefCell::new(None::<SharedSignal<T>>));

        self.effect_scoped(move |cx| {
            let value = f(cx);

            if signal.borrow().is_some() {
                signal.borrow().as_ref().unwrap().set(value);
            } else {
                *signal.borrow_mut() = Some(SharedSignal::new(value));
            }
        });

        signal.borrow().as_ref().unwrap().clone()
    }
}

#[derive(Debug)]
enum ScopeDisposerInner<'a> {
    Root {
        raw: *mut RawScope<'a>,
    },
    Child {
        parent: &'a RawScope<'a>,
        index: usize,
    },
}

#[derive(Debug)]
pub struct ScopeDisposer<'a> {
    inner: ScopeDisposerInner<'a>,
}

impl<'a> ScopeDisposer<'a> {
    fn root(raw: *mut RawScope<'a>) -> Self {
        Self {
            inner: ScopeDisposerInner::Root { raw },
        }
    }

    fn child(parent: &'a RawScope<'a>, index: usize) -> Self {
        Self {
            inner: ScopeDisposerInner::Child { parent, index },
        }
    }

    fn is_drop_locked(&self) -> bool {
        match self.inner {
            ScopeDisposerInner::Root { .. } => false,
            ScopeDisposerInner::Child { parent, index } => parent.is_child_scope_drop_locked(index),
        }
    }

    /// Disposes the scope.
    ///
    /// # Safety
    /// - The scope must not be accessed after calling this method.
    pub unsafe fn dispose(self) {
        match self.inner {
            ScopeDisposerInner::Root { raw } => {
                // SAFETY: `raw` is the only reference to the scope.
                let cx = Box::from_raw(raw);
                cx.dispose();
            }
            ScopeDisposerInner::Child { parent, index } => {
                let mut children = parent.children.borrow_mut();
                let child = children.remove(index).unwrap();
                // SAFETY: `child` is the only reference to the scope.
                let cx = Box::from_raw(child);
                cx.dispose();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn test_signal() {
        Scope::immediate(|cx| {
            let signal = cx.signal(0);

            let cell = &*cx.alloc(Cell::new(0));
            cx.effect(move || {
                cell.set(*signal.get());
            });
            signal.set(1);

            assert_eq!(cell.get(), 1);
        });
    }

    #[test]
    fn test_memo() {
        Scope::immediate(|cx| {
            let signal = cx.signal(0);

            let memo = cx.memo(|| *signal.get() + 1);
            signal.set(1);

            assert_eq!(*memo, 2);
        });
    }
}
