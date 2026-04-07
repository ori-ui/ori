use std::cell::Cell;

thread_local! {
    static RELAXED_TYPE_CHECK: Cell<bool> = const { Cell::new(false) };
}

/// Set whether type checks should use [`type_name`](std::any::type_name) in stead of [`TypeId`].
///
/// This is useful for hot reloading.
///
/// # Safety
///
/// This is inherently unsafe and should **never** be used in production.
pub unsafe fn set_relaxed_type_check(enabled: bool) {
    RELAXED_TYPE_CHECK.set(enabled);
}

/// Check whether relaxed type checking is enabled, see [`set_relaxed_type_check`].
pub fn get_relaxed_type_check() -> bool {
    RELAXED_TYPE_CHECK.get()
}
