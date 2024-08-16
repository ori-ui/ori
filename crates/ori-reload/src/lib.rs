use std::{
    env, fs,
    hash::{DefaultHasher, Hash, Hasher},
    thread::JoinHandle,
    time::SystemTime,
};

use libloading::{Library, Symbol};
use ori_core::{
    command::CommandProxy,
    prelude::{BuildCx, DrawCx, Event, EventCx, LayoutCx, RebuildCx, Size, Space},
    view::View,
};

/// A reloader for shared libraries.
pub struct Reloader {
    libraries: Vec<Library>,
    library_path: &'static str,
    last_update: Option<SystemTime>,
}

impl Reloader {
    /// Create a new reloader.
    pub const fn new(library_path: &'static str) -> Self {
        Self {
            libraries: Vec::new(),
            library_path,
            last_update: None,
        }
    }

    /// Load a library from the library path
    ///
    /// # Safety
    /// - This can never be safe.
    pub unsafe fn load<T>(&mut self, symbol: &[u8]) -> Option<Symbol<T>> {
        let meta = fs::metadata(self.library_path).ok()?;
        let modified = meta.modified().ok()?;

        if self.last_update != Some(modified) {
            self.last_update = Some(modified);

            let mut hasher = DefaultHasher::new();
            modified.hash(&mut hasher);

            let module_name = format!("ori-{}.module", hasher.finish());

            let temp_dir = env::temp_dir();
            let temp_path = temp_dir.join(&module_name);

            fs::copy(self.library_path, &temp_path).ok()?;
            let library = Library::new(&temp_path).ok()?;
            self.libraries.push(library);
        }

        let library = self.libraries.last()?;
        library.get::<T>(symbol).ok()
    }
}

struct Modified;

/// A view that watches a file for changes.
pub struct Watcher<V> {
    /// The content.
    pub content: V,

    /// The file to watch.
    pub file: String,
}

impl<V> Watcher<V> {
    /// Create a new watcher.
    pub fn new(content: V, file: String) -> Self {
        Self { content, file }
    }

    fn spawn(&self, proxy: CommandProxy) -> JoinHandle<()> {
        let file = self.file.clone();
        std::thread::spawn(move || {
            let modified = fs::metadata(&file).ok().and_then(|m| m.modified().ok());

            if modified.is_none() {
                return;
            }

            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));

                let new_modified = fs::metadata(&file).ok().and_then(|m| m.modified().ok());

                if new_modified != modified {
                    proxy.cmd(Modified);
                    break;
                }
            }
        })
    }
}

impl<T, V: View<T>> View<T> for Watcher<V> {
    type State = (V::State, JoinHandle<()>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let handle = self.spawn(cx.proxy());
        let state = self.content.build(cx, data);
        (state, handle)
    }

    fn rebuild(
        &mut self,
        (state, _): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        self.content.rebuild(state, cx, data, &old.content);
    }

    fn event(
        &mut self,
        (state, handle): &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) {
        if event.is_cmd::<Modified>() && handle.is_finished() {
            cx.request_rebuild();

            *handle = self.spawn(cx.proxy());
        }

        self.content.event(state, cx, data, event);
    }

    fn layout(
        &mut self,
        (state, _): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        self.content.layout(state, cx, data, space)
    }

    fn draw(&mut self, (state, _): &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        self.content.draw(state, cx, data);
    }
}
