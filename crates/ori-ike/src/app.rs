use std::any::Any;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

use crate::{Context, Effect};

pub struct App {}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run<T, V>(self, data: &mut T, mut ui: impl FnMut(&mut T) -> V + 'static)
    where
        V: Effect<T> + 'static,
        V::State: 'static,
    {
        let event_loop = EventLoop::new().unwrap();

        let mut build: UiBuilder<T> = Box::new(move |data| Box::new(ui(data)));
        let ui = build(data);

        let mut state = AppState {
            data,
            build,
            ui,
            state: None,
        };

        event_loop.run_app(&mut state).unwrap();
    }
}

struct AppState<'a, T> {
    data: &'a mut T,

    build: UiBuilder<T>,
    ui: AnyEffect<T>,
    state: Option<Box<dyn Any>>,
}

impl<T> ApplicationHandler for AppState<'_, T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let mut cx = Context {
            commands: Vec::new(),
        };

        let (_, mut state) = ori::View::build(&mut self.ui, &mut cx, self.data);

        for command in cx.commands {
            println!("{command:?}");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
    }
}

type AnyEffect<T> = Box<dyn ori::AnyView<Context, T, ori::NoElement>>;
type UiBuilder<T> = Box<dyn FnMut(&mut T) -> AnyEffect<T>>;
