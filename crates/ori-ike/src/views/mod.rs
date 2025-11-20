mod aligned;
mod button;
mod container;
mod label;
mod stack;
mod window;

pub use aligned::{Aligned, align, center};
pub use button::{Button, button};
pub use container::{Container, container};
pub use label::{Label, label};
pub use stack::{Stack, hstack, stack, vstack};
pub use window::{Window, window};

use crate::{Context, View};

pub type AnyView<T> = Box<dyn ori::AnyView<Context, T, ike::WidgetId>>;

#[must_use]
pub fn any<T, V>(view: V) -> AnyView<T>
where
    V: View<T> + 'static,
    V::State: 'static,
{
    Box::new(view)
}
