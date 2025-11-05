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
use ori::View as _;

use crate::{AnyView, Context, Event, Window};

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
    windows: Vec<(u64, Option<Window>, UiBuilder<T>)>,
    css_paths: Vec<PathBuf>,
    css_strings: Vec<String>,
}

impl<T> App<T> {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            css_paths: Vec::new(),
            css_strings: Vec::new(),
        }
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
        let app = gtk4::Application::builder()
            .application_id("com.example.counter")
            .build();

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

                for window in &windows {
                    #[cfg(feature = "layer-shell")]
                    use gtk4_layer_shell::{Edge, LayerShell};

                    let win = gtk4::ApplicationWindow::new(app);

                    #[cfg(feature = "layer-shell")]
                    if let Some(layer) = window.layer {
                        win.init_layer_shell();
                        win.set_layer(layer.into());

                        if let Some(zone) = window.exclusive_zone {
                            win.set_exclusive_zone(zone);
                        } else {
                            win.auto_exclusive_zone_enable();
                        }

                        win.set_anchor(Edge::Top, window.anchor_top);
                        win.set_anchor(Edge::Right, window.anchor_right);
                        win.set_anchor(Edge::Bottom, window.anchor_bottom);
                        win.set_anchor(Edge::Left, window.anchor_left);

                        win.set_margin(Edge::Top, window.margin_top);
                        win.set_margin(Edge::Right, window.margin_right);
                        win.set_margin(Edge::Bottom, window.margin_bottom);
                        win.set_margin(Edge::Left, window.margin_left);
                    }

                    win.set_title(Some(&window.title));
                    win.set_default_size(
                        window.width as i32,
                        window.height as i32,
                    );

                    context.opened(window.id);
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
                                    sender.send(Event::StyleChanged(canonical));
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
                    if let Err(err) =
                        state.handle_event(&mut self, &mut data, event)
                    {
                        println!("error: {err}");
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
    fn add_css_path(&mut self, path: &Path) -> Result<(), Error> {
        let canonical = fs::canonicalize(path)?;

        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_path(&canonical);

        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().unwrap(),
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        self.watcher.watch(
            &path,
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
                for path in &app.css_paths {
                    self.add_css_path(path)?;
                }

                for string in &app.css_strings {
                    let css_provider = gtk4::CssProvider::new();
                    css_provider.load_from_data(string);

                    gtk4::style_context_add_provider_for_display(
                        &gtk4::gdk::Display::default().unwrap(),
                        &css_provider,
                        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
                    );
                }
            }

            Event::StyleChanged(path) => {
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
                        window.view.as_mut(),
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
                window.show();

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
            }

            Event::WindowClosed(id) => {
                let index = self.windows.iter().position(|w| w.id == id);

                if let Some(index) = index {
                    let mut window = self.windows.swap_remove(index);

                    window.view.teardown(
                        &mut window.element,
                        &mut window.state,
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

                    if action.rebuild {
                        self.context.rebuild();
                    }
                }
            }
        }

        Ok(true)
    }
}
