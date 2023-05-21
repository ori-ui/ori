mod callback;
mod emitter;

pub use callback::*;
pub use emitter::*;

type RawCallback<'a, T> = dyn FnMut(&T) + Send + 'a;
type CallbackPtr<T> = *const T;
