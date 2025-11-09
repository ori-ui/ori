use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
    time::Duration,
};

use futures_util::StreamExt;
use gtk4::{
    gio::prelude::{ApplicationExt as _, ApplicationExtManual as _},
    glib::clone::Downgrade as _,
};
use notify::Watcher as _;
use ori::{AsyncContext as _, View as _};

use crate::{AnyEffect, Context, context::Event};

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

pub struct App {
    id: Option<String>,
    theme: Option<String>,
    css_paths: Vec<PathBuf>,
    css_strings: Vec<String>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            id: None,
            theme: None,
            css_paths: Vec::new(),
            css_strings: Vec::new(),
        }
    }

    pub fn id(mut self, id: impl ToString) -> Self {
        self.id = Some(id.to_string());
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

    pub fn run<T, V>(
        mut self,
        data: T,
        mut ui: impl FnMut(&mut T) -> V + 'static,
    ) -> Result<ExitCode, Error>
    where
        T: 'static,
        V: ori::View<Context, T, Element = ori::NoElement> + 'static,
    {
        let ui: UiBuilder<T> = Box::new(move |data| Box::new(ui(data)));

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
        let (win_tx, mut win_rx) = futures_channel::mpsc::unbounded();

        app.connect_activate({
            let context = context.clone();
            let data = RefCell::new(Some(data));
            let ui = RefCell::new(Some(ui));

            move |_app| {
                let mut cx = context.clone();
                let mut data = data.borrow_mut().take().unwrap();
                let mut ui = ui.borrow_mut().take().unwrap();

                let mut view = ui(&mut data);
                let (_, state) = view.build(&mut cx, &mut data);

                win_tx.unbounded_send((data, ui, view, state)).unwrap();
            }
        });

        let main_context = gtk4::glib::MainContext::default();

        main_context.clone().spawn_local({
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
                                let _ = sender.unbounded_send(Event::CssChanged(canonical));
                            }
                        }
                    }
                },
                config,
            )?;

            async move {
                let (mut data, builder, view, state) = win_rx.next().await.unwrap();

                let mut state = AppState {
                    main_context,
                    watcher,
                    builder,
                    view,
                    state,
                    context: context.clone(),
                    css_paths: HashMap::new(),
                };

                if let Err(err) = state.activate(&mut self) {
                    eprintln!("error: {err}");
                }

                while let Some(event) = receiver.next().await {
                    let result = state.handle_event(&mut data, event);

                    if let Err(err) = result {
                        eprintln!("error: {err}");
                    }
                }
            }
        });

        Ok(app.run_with_args::<&str>(&[]).into())
    }
}

type UiBuilder<T> = Box<dyn FnMut(&mut T) -> AnyEffect<T>>;

struct AppState<T> {
    main_context: gtk4::glib::MainContext,
    builder: UiBuilder<T>,
    view: AnyEffect<T>,
    state: Box<dyn Any>,
    watcher: notify::PollWatcher,
    context: Context,
    css_paths: HashMap<PathBuf, gtk4::CssProvider>,
}

impl<T> AppState<T> {
    fn activate(&mut self, app: &mut App) -> Result<(), Error> {
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

        Ok(())
    }

    fn add_css_path(&mut self, path: &Path, display: &gtk4::gdk::Display) -> Result<(), Error> {
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

    fn handle_event(&mut self, data: &mut T, event: Event) -> Result<bool, Error> {
        match event {
            Event::CssChanged(path) => {
                if let Some(provider) = self.css_paths.get(&path) {
                    provider.load_from_path(path);
                }
            }

            Event::Rebuild => {
                let mut view = (self.builder)(data);

                view.rebuild(
                    &mut ori::NoElement,
                    &mut self.state,
                    &mut self.context,
                    data,
                    &mut self.view,
                );

                self.view = view;
            }

            Event::Event(mut event) => {
                let (_, action) = self.view.event(
                    &mut ori::NoElement,
                    &mut self.state,
                    &mut self.context,
                    data,
                    &mut event,
                );

                self.context.send_action(action);
            }

            Event::Spawn(future) => {
                self.main_context.spawn_local(future);
            }
        }

        Ok(true)
    }
}
