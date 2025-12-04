use std::{
    any::Any,
    cell::RefCell,
    env,
    ffi::CStr,
    fs, mem,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::Duration,
};

use libloading::{Library, Symbol};
use ori::AsyncContext;
use rand::RngCore;

pub use ori;

use crate::Context;

#[derive(Clone, Copy)]
pub struct Environment {
    pub cargo:        &'static str,
    pub manifest_dir: &'static str,
    pub package_name: &'static str,
}

#[macro_export]
macro_rules! reload {
    ($ui:expr) => {{
        unsafe fn reload() -> $crate::reload::Reload {
            #[unsafe(no_mangle)]
            extern "Rust" fn reload(
                data: &mut dyn ::std::any::Any,
            ) -> ::std::boxed::Box<
                dyn $crate::AnyView<$crate::Context, dyn ::std::any::Any, $crate::WidgetId>,
            > {
                let ui: fn(&mut _) -> _ = $ui;

                ::std::boxed::Box::new($crate::views::map(
                    ui(unsafe { &mut *(data as *mut _ as *mut _) }),
                    |data: &mut dyn ::std::any::Any, map| {
                        map(unsafe { &mut *(data as *mut _ as *mut _) })
                    },
                ))
            }

            let env = $crate::reload::Environment {
                cargo:        ::std::env!("CARGO"),
                manifest_dir: ::std::env!("CARGO_MANIFEST_DIR"),
                package_name: ::std::env!("CARGO_PKG_NAME"),
            };

            unsafe { $crate::reload::Reload::new(reload, c"reload", env) }
        }

        reload()
    }};
}

fn lib_path(env: Environment) -> PathBuf {
    let output = Command::new(env.cargo)
        .current_dir(env.manifest_dir)
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .output()
        .unwrap();

    let meta = String::from_utf8_lossy(&output.stdout);
    let meta = json::parse(&meta).unwrap();
    let target_dir = meta["target_directory"].as_str().unwrap();

    Path::new(target_dir)
        .join(env!("TARGET"))
        .join(env!("PROFILE"))
        .join(format!("lib{}.so", env.package_name))
}

fn build(env: Environment) -> Result<PathBuf, ()> {
    let output = Command::new(env.cargo)
        .current_dir(env.manifest_dir)
        .arg("build")
        .arg("--lib")
        .arg("--target")
        .arg(env!("TARGET"))
        .arg("--package")
        .arg(env.package_name)
        .args(if env!("PROFILE") == "release" {
            &["--release"] as &[_]
        } else {
            &[]
        })
        .output()
        .unwrap();

    if !output.status.success() {
        return Err(());
    }

    Ok(lib_path(env))
}

fn watcher(env: Environment, proxy: impl ori::Proxy) {
    thread::spawn(move || {
        let _ = build(env);

        let mut modified = lib_path(env).metadata().unwrap().modified().unwrap();

        loop {
            thread::sleep(Duration::from_secs_f32(0.5));

            if build(env).is_ok() {
                let lib_path = lib_path(env);
                let current = lib_path.metadata().unwrap().modified().unwrap();

                if current > modified {
                    modified = current;

                    proxy.event(ori::Event::new(
                        ReloadEvent::Reload(lib_path),
                        None,
                    ));
                }
            }
        }
    });
}

type Build =
    extern "Rust" fn(&mut dyn Any) -> Box<dyn ori::AnyView<Context, dyn Any, ike::WidgetId>>;

pub struct Reload {
    #[allow(clippy::type_complexity)]
    initial: Build,
    symbol:  &'static CStr,
    env:     Environment,
}

impl Reload {
    /// # Safety
    /// - Calling this can never be safe, and must only be used for debugging.
    #[allow(clippy::type_complexity)]
    pub unsafe fn new(initial: Build, symbol: &'static CStr, env: Environment) -> Self {
        Self {
            initial,
            symbol,
            env,
        }
    }
}

enum ReloadEvent {
    Reload(PathBuf),
}

pub struct ReloadState {
    view:  Box<dyn ori::AnyView<Context, dyn Any, ike::WidgetId>>,
    state: Box<dyn Any>,
    build: Build,
}

thread_local! {
    static LIBS: RefCell<Vec<Library>> = Default::default();
}

impl ori::ViewMarker for Reload {}
impl<T> ori::View<Context, T> for Reload
where
    T: 'static,
{
    type Element = ike::WidgetId;
    type State = ReloadState;

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let mut view = (self.initial)(data);

        let use_type_names = cx.use_type_names_unsafe;
        cx.use_type_names_unsafe = true;
        let (element, state) = view.build(cx, data);
        cx.use_type_names_unsafe = use_type_names;

        watcher(self.env, cx.proxy());

        let state = ReloadState {
            view,
            state,
            build: self.initial,
        };

        (element, state)
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        _old: &mut Self,
    ) {
        let use_type_names = cx.use_type_names_unsafe;
        cx.use_type_names_unsafe = true;

        let mut new_view = (state.build)(data);

        new_view.rebuild(
            element,
            &mut state.state,
            cx,
            data,
            &mut state.view,
        );
        state.view = new_view;

        cx.use_type_names_unsafe = use_type_names;
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        mut state: Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        let use_type_names = cx.use_type_names_unsafe;
        cx.use_type_names_unsafe = true;

        state.view.teardown(element, state.state, cx, data);

        cx.use_type_names_unsafe = use_type_names;
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        let use_type_names = cx.use_type_names_unsafe;
        cx.use_type_names_unsafe = true;

        match event.get() {
            Some(ReloadEvent::Reload(output)) => {
                let temp_dir = env::temp_dir();

                let mut source = rand::rng();
                let lib = temp_dir.join(format!(
                    "lib{}-{:08x}.so",
                    self.env.package_name,
                    source.next_u64(),
                ));

                fs::copy(output, &lib).unwrap();

                let library = unsafe { Library::new(&lib).unwrap() };
                let builder: Symbol<Build> = unsafe { library.get(self.symbol).unwrap() };

                let mut new_view = builder(data);
                let (new_element, new_state) = new_view.build(cx, data);

                let mut old_view = mem::replace(&mut state.view, new_view);
                let old_element = mem::replace(element, new_element);
                let old_state = mem::replace(&mut state.state, new_state);

                old_view.teardown(old_element, old_state, cx, data);

                state.build = *builder;

                LIBS.with_borrow_mut(|libs| libs.push(library));

                tracing::debug!("reloading");
            }

            None => {}
        }

        let action = state.view.event(
            element,
            &mut state.state,
            cx,
            data,
            event,
        );

        cx.use_type_names_unsafe = use_type_names;

        action
    }
}
