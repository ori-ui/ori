#![deny(missing_docs)]
#![allow(clippy::module_inception)]

//! Ori [`reload`](ori_reload) module.

use std::{
    env, fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::Path,
    process::Command,
    sync::Once,
    thread::{self, JoinHandle},
    time::SystemTime,
};

use libloading::{Library, Symbol};
use ori_core::{
    command::CommandProxy,
    prelude::{BuildCx, DrawCx, Event, EventCx, LayoutCx, RebuildCx, Size, Space},
    view::View,
};

fn modified(path: impl AsRef<Path>) -> Option<SystemTime> {
    fs::metadata(path).ok()?.modified().ok()
}

fn modified_hasher(hasher: &mut DefaultHasher, path: impl AsRef<Path>) {
    match fs::read_dir(&path) {
        Ok(dir) => {
            for entry in dir.flatten() {
                modified_hasher(hasher, entry.path());
            }
        }
        Err(_) => modified(path).hash(hasher),
    }
}

fn modified_hash(path: impl AsRef<Path>) -> u64 {
    let mut hasher = DefaultHasher::new();
    modified_hasher(&mut hasher, path);
    hasher.finish()
}

/// Start a cargo build watcher.
pub fn start_cargo_build_watcher(manifest_dir: &str, reload_feature: &str, is_release: bool) {
    static WATCHER: Once = Once::new();

    WATCHER.call_once(|| {
        let dir = Path::new(manifest_dir).join("src");
        let feature = reload_feature.to_string();

        thread::spawn(move || {
            let mut last_update = modified_hash(&dir);

            loop {
                let new_modified = modified_hash(&dir);

                if last_update != new_modified {
                    last_update = new_modified;

                    let mut command = Command::new("cargo");

                    command
                        .current_dir(&dir)
                        .arg("build")
                        .arg("--lib")
                        .arg("--features")
                        .arg(&feature);

                    if is_release {
                        command.arg("--release");
                    }

                    command.status().unwrap();
                }

                thread::sleep(std::time::Duration::from_secs(1));
            }
        });
    });
}

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
        thread::spawn(move || {
            let last_modified = modified(&file);

            if last_modified.is_none() {
                return;
            }

            loop {
                thread::sleep(std::time::Duration::from_secs(1));

                let new_modified = modified(&file);
                if new_modified != last_modified {
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
