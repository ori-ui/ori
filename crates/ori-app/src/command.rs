use ori_core::{
    view::{BoxedView, View},
    window::{Window, WindowId},
};

/// Commands that can be sent to the application.
///
/// # Example
/// ```no_run
/// # use ori_core::{view::View, views::*, context::*};
/// # use crate::AppCommand;
/// fn ui() -> impl View {
///     // Here we create a button that quits the application when clicked.
///     on_click(
///         button(text("Quit")),
///         |cx, _| cx.cmd(AppCommand::Quit),
///     )
/// }
/// ```
pub enum AppCommand {
    /// Open a new window.
    OpenWindow(Window, Box<dyn FnMut() -> BoxedView<()> + Send>),

    /// Close a window.
    CloseWindow(WindowId),

    /// Drag a window.
    DragWindow(WindowId),

    /// Quit the application.
    Quit,
}

impl AppCommand {
    /// Convenience method to open a window with a view.
    ///
    /// Note that `V` must implement `View<()>`, and therefore cannot access the data of the
    /// application.
    ///
    /// # Example
    /// ```no_run
    /// # use ori_core::{view::View, views::*, context::*};
    /// # use crate::AppCommand;
    /// fn ui() -> impl View {
    ///     // Here we create a button that opens a new window when clicked.
    ///     on_click(
    ///         button(text("Open new window")),
    ///         |cx, _| {
    ///             let window = Window::new()
    ///                 .title("New window");
    ///
    ///             cx.cmd(AppCommand::open_window(window, popup));
    ///         },
    ///     )
    /// }
    ///
    /// fn popup() -> impl View {
    ///     text("Hello, world!")
    /// }
    /// ```
    pub fn open_window<V: View + 'static>(
        window: Window,
        mut view: impl FnMut() -> V + Send + 'static,
    ) -> Self {
        Self::OpenWindow(window, Box::new(move || Box::new(view())))
    }
}
