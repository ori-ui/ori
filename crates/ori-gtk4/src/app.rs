use std::{
    any::Any,
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
    sync::mpsc::Receiver,
    time::Duration,
};

use gtk4::{
    gio::prelude::{ApplicationExt as _, ApplicationExtManual as _},
    glib::clone::Downgrade as _,
    prelude::{GtkWindowExt as _, WidgetExt as _},
};
use notify::Watcher as _;
use ori::{AsyncContext as _, Proxy as _, View as _};

use crate::{AnyView, Context, Window, WindowEvent, context::Event};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Notify(#[from] notify::Error),

    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Css {
    Path(PathBuf),
    String(String),
}

/// On debug builds load CSS from the file, and watch for changes.
///
/// On release builds include the CSS into the binary by string.
#[macro_export]
macro_rules! include_css {
    ($path:literal) => {{
        #[allow(dead_code)]
        let included = ::std::include_str!($path);

        #[cfg(not(debug_assertions))]
        let css = $crate::Css::String(::std::convert::From::from(included));

        #[cfg(debug_assertions)]
        let css = {
            let file = ::std::path::Path::new(::std::file!());
            $crate::Css::Path(file.parent().unwrap().join($path))
        };

        css
    }};
}

pub struct App<T> {
    id: Option<String>,
    windows: Vec<(u64, Option<Window>, UiBuilder<T>)>,
    theme: Option<String>,
    css_paths: Vec<PathBuf>,
    css_strings: Vec<String>,
}

impl<T> Default for App<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> App<T> {
    pub fn new() -> Self {
        Self {
            id: None,
            windows: Vec::new(),
            theme: None,
            css_paths: Vec::new(),
            css_strings: Vec::new(),
        }
    }

    pub fn id(mut self, id: impl ToString) -> Self {
        self.id = Some(id.to_string());
        self
    }

    pub fn window<V>(
        mut self,
        window: Window,
        mut ui: impl FnMut(&mut T) -> V + 'static,
    ) -> Self
    where
        V: ori::AnyView<Context, gtk4::Widget, T> + 'static,
    {
        let builder: UiBuilder<T> = Box::new(move |data| Box::new(ui(data)));
        self.windows.push((window.id, Some(window), builder));
        self
    }

    pub fn theme(mut self, theme: impl ToString) -> Self {
        self.theme = Some(theme.to_string());
        self
    }

    pub fn css(self, css: Css) -> Self {
        match css {
            Css::Path(path) => self.css_path(path),
            Css::String(string) => self.css_string(string),
        }
    }

    pub fn css_path(mut self, path: impl AsRef<Path>) -> Self {
        self.css_paths.push(path.as_ref().to_path_buf());
        self
    }

    pub fn css_string(mut self, string: impl ToString) -> Self {
        self.css_strings.push(string.to_string());
        self
    }

    pub fn run(mut self, mut data: T) -> Result<ExitCode, Error>
    where
        T: 'static,
    {
        gtk4::init().unwrap();

        if let Some(ref theme) = self.theme
            && let Some(display) = gtk4::gdk::Display::default()
        {
            let settings = gtk4::Settings::for_display(&display);
            settings.set_gtk_theme_name(Some(theme));
        }

        let app = gtk4::Application::new(
            self.id.as_deref(),
            gtk4::gio::ApplicationFlags::empty(),
        );

        let (context, mut receiver) = Context::new(app.downgrade());
        let (win_tx, win_rx) = std::sync::mpsc::channel();

        app.connect_activate({
            let context = context.clone();
            let windows = self
                .windows
                .iter_mut()
                .map(|(_, w, _)| w.take().unwrap())
                .collect::<Vec<_>>();

            move |app| {
                context.activate();

                for desc in &windows {
                    #[cfg(feature = "layer-shell")]
                    use gtk4_layer_shell::{Edge, LayerShell};

                    let win = gtk4::ApplicationWindow::new(app);

                    #[cfg(feature = "layer-shell")]
                    if desc.is_layer_shell {
                        win.init_layer_shell();
                        win.set_layer(desc.layer.into());

                        if let Some(zone) = desc.exclusive_zone {
                            win.set_exclusive_zone(zone);
                        } else {
                            win.auto_exclusive_zone_enable();
                        }

                        win.set_anchor(Edge::Top, desc.anchor_top);
                        win.set_anchor(Edge::Right, desc.anchor_right);
                        win.set_anchor(Edge::Bottom, desc.anchor_bottom);
                        win.set_anchor(Edge::Left, desc.anchor_left);

                        win.set_margin(Edge::Top, desc.margin_top);
                        win.set_margin(Edge::Right, desc.margin_right);
                        win.set_margin(Edge::Bottom, desc.margin_bottom);
                        win.set_margin(Edge::Left, desc.margin_left);
                    }

                    win.set_title(Some(&desc.title));
                    win.set_default_size(
                        desc.width.map_or(-1, |width| width as i32),
                        desc.height.map_or(-1, |width| width as i32),
                    );

                    win.set_resizable(desc.resizable);
                    win.set_decorated(desc.decorated);
                    win.set_focus_visible(desc.show_focus);
                    win.set_hide_on_close(desc.hide_on_close);

                    context.opened(desc.id);
                    win_tx.send(win).unwrap();
                }
            }
        });

        gtk4::glib::MainContext::default().spawn_local({
            let config = notify::Config::default()
                .with_poll_interval(Duration::from_secs(1))
                .with_compare_contents(true);

            let watcher = notify::PollWatcher::new(
                {
                    let sender = context.sender().clone();

                    move |event: notify::Result<notify::Event>| {
                        let Ok(event) = event else { return };

                        if !event.kind.is_modify() {
                            return;
                        }

                        for path in event.paths {
                            if let Ok(canonical) = fs::canonicalize(path) {
                                let _ =
                                    sender.send(Event::CssChanged(canonical));
                            }
                        }
                    }
                },
                config,
            )?;

            let mut state = AppState {
                win_rx,
                watcher,
                context: context.clone(),
                windows: Vec::new(),
                css_paths: HashMap::new(),
            };

            async move {
                while let Some(event) = receiver.recv().await {
                    let result = state.handle_event(
                        &mut self, &mut data, event, //
                    );

                    if let Err(err) = result {
                        eprintln!("error: {err}");
                    }
                }
            }
        });

        Ok(app.run().into())
    }
}

type UiBuilder<T> = Box<dyn FnMut(&mut T) -> AnyView<T>>;

struct WindowState<T> {
    id: u64,
    window: gtk4::ApplicationWindow,
    builder: UiBuilder<T>,
    view: AnyView<T>,
    element: gtk4::Widget,
    state: Box<dyn Any>,
}

struct AppState<T> {
    win_rx: Receiver<gtk4::ApplicationWindow>,
    watcher: notify::PollWatcher,
    context: Context,
    windows: Vec<WindowState<T>>,
    css_paths: HashMap<PathBuf, gtk4::CssProvider>,
}

impl<T> AppState<T> {
    fn add_css_path(
        &mut self,
        path: &Path,
        display: &gtk4::gdk::Display,
    ) -> Result<(), Error> {
        let canonical = fs::canonicalize(path)?;

        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_path(&canonical);

        gtk4::style_context_add_provider_for_display(
            display,
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        self.watcher.watch(
            path,
            notify::RecursiveMode::NonRecursive,
        )?;

        self.css_paths.insert(canonical, css_provider);

        Ok(())
    }

    fn handle_event(
        &mut self,
        app: &mut App<T>,
        data: &mut T,
        event: Event,
    ) -> Result<bool, Error> {
        match event {
            Event::Activate => {
                let display = gtk4::gdk::Display::default().unwrap();

                for path in &app.css_paths {
                    self.add_css_path(path, &display)?;
                }

                for string in &app.css_strings {
                    let css_provider = gtk4::CssProvider::new();
                    css_provider.load_from_data(string);

                    gtk4::style_context_add_provider_for_display(
                        &display,
                        &css_provider,
                        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
                    );
                }
            }

            Event::CssChanged(path) => {
                if let Some(provider) = self.css_paths.get(&path) {
                    provider.load_from_path(path);
                }
            }

            Event::Rebuild => {
                for window in &mut self.windows {
                    let mut view = (window.builder)(data);

                    view.rebuild(
                        &mut window.element,
                        &mut window.state,
                        &mut self.context,
                        data,
                        &mut window.view,
                    );

                    if !window.element.is_ancestor(&window.window) {
                        window.window.set_child(Some(&window.element));
                    }

                    window.view = view;
                }
            }

            Event::InitialWindowCreated(id) => {
                let index = app
                    .windows
                    .iter()
                    .position(|(wid, _, _)| *wid == id)
                    .unwrap();

                let (_, _, mut builder) = app.windows.swap_remove(index);

                let mut view = builder(data);
                let (element, state) = view.build(&mut self.context, data);

                let window = self.win_rx.recv().unwrap();

                window.set_child(Some(&element));
                window.present();

                window.connect_close_request({
                    let context = self.context.clone();

                    move |_| {
                        context.closed(id);
                        gtk4::glib::Propagation::Proceed
                    }
                });

                let window_state = WindowState {
                    id,
                    window,
                    builder,
                    view,
                    element,
                    state,
                };

                self.windows.push(window_state);

                self.context.event(WindowEvent::Activate, None);
            }

            Event::WindowClosed(id) => {
                let index = self.windows.iter().position(|w| w.id == id);

                if let Some(index) = index {
                    let mut window = self.windows.swap_remove(index);

                    window.view.teardown(
                        window.element,
                        window.state,
                        &mut self.context,
                        data,
                    );
                }
            }

            Event::Event(mut event) => {
                for window in &mut self.windows {
                    let action = window.view.event(
                        &mut window.element,
                        &mut window.state,
                        &mut self.context,
                        data,
                        &mut event,
                    );

                    self.context.proxy().action(action);
                }
            }

            Event::Spawn(future) => {
                gtk4::glib::MainContext::default().spawn(future);
            }
        }

        Ok(true)
    }
}
