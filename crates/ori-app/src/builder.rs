use std::{
    convert::Infallible,
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, SystemTime},
};

use ori_core::{
    canvas::{BorderRadius, BorderWidth},
    command::{CommandProxy, CommandWaker},
    context::Contexts,
    layout::{Align, Justify, Padding},
    style::{Styles, Theme},
    text::{
        include_font, FontFamily, FontSource, FontStretch, FontStyle, FontWeight, Fonts, TextAlign,
        TextWrap,
    },
    window::Window,
};

use crate::{App, AppCommand, AppDelegate, AppRequest, IntoUiBuilder};

/// Include a style file.
#[macro_export]
macro_rules! include_style {
    ($path:literal) => {{
        $crate::StyleFileLoader::new(
            ::std::concat!(::std::env!("CARGO_MANIFEST_DIR"), "/", $path),
            ::std::include_str!(::std::concat!(
                ::std::env!("CARGO_MANIFEST_DIR"),
                "/",
                $path,
            )),
        )
    }};
}

/// A builder for an [`App`].
pub struct AppBuilder<T> {
    delegates: Vec<Box<dyn AppDelegate<T>>>,
    requests: Vec<AppRequest<T>>,
    style_loader: StyleLoader,
    fonts: Vec<FontSource<'static>>,
}

impl<T> Default for AppBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> AppBuilder<T> {
    /// Create a new application builder.
    pub fn new() -> Self {
        let mut styles = Styles::from(Theme::dark());

        styles.add_conversion::<f32, _>(BorderWidth::from);
        styles.add_conversion::<[f32; 2], _>(BorderWidth::from);
        styles.add_conversion::<[f32; 4], _>(BorderWidth::from);

        styles.add_conversion::<f32, _>(BorderRadius::from);
        styles.add_conversion::<[f32; 4], _>(BorderRadius::from);

        styles.add_conversion::<f32, _>(Padding::from);
        styles.add_conversion::<[f32; 2], _>(Padding::from);
        styles.add_conversion::<[f32; 4], _>(Padding::from);

        styles.add_conversion::<String, _>(FontFamily::from);
        styles.add_conversion::<String, _>(FontWeight::from);
        styles.add_conversion::<String, _>(FontStretch::from);
        styles.add_conversion::<String, _>(FontStyle::from);
        styles.add_conversion::<String, _>(TextAlign::from);
        styles.add_conversion::<String, _>(TextWrap::from);

        styles.add_conversion::<String, _>(Align::from);
        styles.add_conversion::<String, _>(Justify::from);

        let mut style_loader = StyleLoader::new();

        style_loader.push(styles);

        Self {
            delegates: Vec::new(),
            requests: Vec::new(),
            style_loader,
            fonts: vec![include_font!("font")],
        }
    }

    /// Add a delegate to the application.
    pub fn delegate(mut self, delegate: impl AppDelegate<T> + 'static) -> Self {
        self.delegates.push(Box::new(delegate));
        self
    }

    /// Add a style to the application.
    pub fn style<L>(mut self, loader: L) -> Self
    where
        L: LoadStyle + 'static,
        L::Err: Error + 'static,
    {
        self.style_loader.push(loader);
        self
    }

    /// Add a font to the application.
    pub fn font(mut self, font: impl Into<FontSource<'static>>) -> Self {
        self.fonts.push(font.into());
        self
    }

    /// Add a window to the application.
    pub fn window<V, P>(
        mut self,
        window: Window,
        builder: impl IntoUiBuilder<V, P, Data = T>,
    ) -> Self {
        let builder = builder.into_ui_builder();
        (self.requests).push(AppRequest::OpenWindow(window, builder));
        self
    }

    /// Build the application.
    pub fn build(self, waker: CommandWaker, mut fonts: Box<dyn Fonts>) -> App<T> {
        for font in self.fonts {
            fonts.load(font, None);
        }

        let (proxy, receiver) = CommandProxy::new(waker);

        let styles = self.style_loader.load().unwrap();
        self.style_loader.spawn_file_watchers(&proxy);

        let mut contexts = Contexts::new();
        contexts.insert(styles);
        contexts.insert(fonts);

        App {
            windows: Default::default(),
            modifiers: Default::default(),
            delegates: self.delegates,
            proxy,
            receiver,
            requests: self.requests,
            contexts,
            style_loader: self.style_loader,
        }
    }
}

/// A trait for loading styles.
pub trait LoadStyle {
    /// The error type.
    type Err;

    /// Load a style.
    fn load_style(&self) -> Result<Styles, Self::Err>;

    /// Watch files for changes.
    fn watch_files(&self) -> Vec<PathBuf> {
        Vec::new()
    }
}

impl LoadStyle for Styles {
    type Err = Infallible;

    fn load_style(&self) -> Result<Styles, Self::Err> {
        Ok(self.clone())
    }
}

impl LoadStyle for Theme {
    type Err = Infallible;

    fn load_style(&self) -> Result<Styles, Self::Err> {
        Ok(From::from(*self))
    }
}

impl LoadStyle for str {
    type Err = ori_core::style::ParseError;

    fn load_style(&self) -> Result<Styles, Self::Err> {
        FromStr::from_str(self)
    }
}

impl LoadStyle for Path {
    type Err = io::Error;

    fn load_style(&self) -> Result<Styles, Self::Err> {
        let s = fs::read_to_string(self)?;
        FromStr::from_str(&s).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

impl LoadStyle for PathBuf {
    type Err = io::Error;

    fn load_style(&self) -> Result<Styles, Self::Err> {
        let s = fs::read_to_string(self)?;
        FromStr::from_str(&s).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

/// A style loader for a single file.
pub struct StyleFileLoader {
    path: PathBuf,
    default: String,
}

impl StyleFileLoader {
    /// Create a new style file loader.
    pub fn new(path: impl Into<PathBuf>, default: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            default: default.into(),
        }
    }
}

impl LoadStyle for StyleFileLoader {
    type Err = io::Error;

    fn load_style(&self) -> Result<Styles, Self::Err> {
        if self.path.exists() {
            return self.path.load_style();
        }

        self.default
            .load_style()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn watch_files(&self) -> Vec<PathBuf> {
        vec![self.path.clone()]
    }
}

type BoxedLoader<T> = Box<dyn LoadStyle<Err = T>>;

/// A style loader, which can load styles from multiple sources.
pub struct StyleLoader {
    loaders: Vec<BoxedLoader<Box<dyn Error>>>,
    is_active: Arc<AtomicBool>,
}

impl Default for StyleLoader {
    fn default() -> Self {
        Self {
            loaders: Vec::new(),
            is_active: Arc::new(AtomicBool::new(true)),
        }
    }
}

impl StyleLoader {
    /// Create a new style loader.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a style loader.
    pub fn push<L>(&mut self, loader: L)
    where
        L: LoadStyle + 'static,
        L::Err: Error + 'static,
    {
        self.loaders.push(Box::new(LoadStyleError(loader)));
    }

    /// Load the styles.
    pub fn load(&self) -> Result<Styles, Box<dyn Error>> {
        let mut styles = Styles::default();

        for loader in &self.loaders {
            styles.extend(loader.load_style()?);
        }

        Ok(styles)
    }

    /// Watch files for changes.
    pub fn watch_files(&self) -> Vec<PathBuf> {
        self.loaders
            .iter()
            .flat_map(|loader| loader.watch_files())
            .collect()
    }

    /// Spawn file watchers for the loaders.
    pub fn spawn_file_watchers(&self, proxy: &CommandProxy) {
        fn get_modified(path: &Path) -> Option<SystemTime> {
            path.metadata().ok()?.modified().ok()
        }

        for path in self.watch_files() {
            let proxy = proxy.clone();
            let is_active = self.is_active.clone();
            let mut last_modified = get_modified(&path);

            thread::spawn(move || {
                while is_active.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_secs(1));

                    let modified = get_modified(&path);

                    if last_modified != modified {
                        last_modified = modified;

                        proxy.cmd(AppCommand::ReloadStyles);
                    }
                }
            });
        }
    }
}

impl Drop for StyleLoader {
    fn drop(&mut self) {
        self.is_active.store(false, Ordering::Relaxed);
    }
}

struct LoadStyleError<T>(T);

impl<T: LoadStyle> LoadStyle for LoadStyleError<T>
where
    T::Err: Error + 'static,
{
    type Err = Box<dyn Error>;

    fn load_style(&self) -> Result<Styles, Self::Err> {
        let Self(ref loader) = self;

        loader.load_style().map_err(|e| Box::new(e) as _)
    }

    fn watch_files(&self) -> Vec<PathBuf> {
        let Self(ref loader) = self;

        loader.watch_files()
    }
}
