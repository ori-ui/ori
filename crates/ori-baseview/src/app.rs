use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use baseview::{EventStatus, MouseEvent, Size, Window, WindowHandler, WindowScalePolicy};
use old_raw_window_handle::HasRawWindowHandle;
use ori_core::{
    KeyboardEvent, LoadedStyleKind, Modifiers, Node, PointerEvent, RootNode, StyleLoader,
    Stylesheet, Vec2, View, WindowResizeEvent,
};
use ori_graphics::{Color, ImageData, ImageSource};
use ori_reactive::{CallbackEmitter, Event, EventEmitter, EventSink, Scope, Task};

use crate::{
    convert::{
        convert_key, convert_modifiers, convert_mouse_button, convert_point, convert_scroll_delta,
        is_pressed,
    },
    window_handle::OldWindowHandle,
};

/// A app using [`winit`] as the windowing backend.
pub struct App {
    title: String,
    size: Vec2,
    reziseable: bool,
    clear_color: Color,
    style_loader: StyleLoader,
    icon: Option<ImageData>,
    builder: Option<Box<dyn FnOnce(&EventSink, &CallbackEmitter<Event>) -> RootNode>>,
}

impl App {
    /// Create a new [`App`] with the given content.
    pub fn new<T: View>(content: impl FnOnce(Scope) -> T + 'static) -> Self {
        let builder = Box::new(
            move |event_sink: &EventSink, callback_emitter: &CallbackEmitter<Event>| {
                let scope = Scope::new(event_sink.clone(), callback_emitter.clone());
                let node = Node::new(content(scope));
                RootNode::new(node, event_sink.clone(), callback_emitter.clone())
            },
        );

        Self {
            title: String::from("Ori App"),
            size: Vec2::new(800.0, 600.0),
            reziseable: false,
            clear_color: Color::WHITE,
            style_loader: StyleLoader::new(),
            icon: None,
            builder: Some(builder),
        }
    }

    /// Set the default theme to night theme, this will clear all the styles
    /// that have been added before, and should therefore be called before
    /// [`App::style`].
    pub fn night_theme(mut self) -> Self {
        self.style_loader.clear();
        self.style_loader
            .add_style(Stylesheet::night_theme())
            .unwrap();
        self
    }

    /// Set the default theme to day theme, this will clear all the styles
    /// that have been added before, and should therefore be called before
    /// [`App::style`].
    pub fn day_theme(mut self) -> Self {
        self.style_loader.clear();
        self.style_loader
            .add_style(Stylesheet::day_theme())
            .unwrap();
        self
    }

    /// Set the title of the window.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Add a style to the app, this can be called multiple times to add
    /// multiple styles.
    pub fn style<T>(mut self, style: T) -> Self
    where
        T: TryInto<LoadedStyleKind>,
        T::Error: Display,
    {
        match self.style_loader.add_style(style) {
            Err(err) => eprintln!("failed to load style: {}", err),
            _ => {}
        };

        self
    }

    /// Set the size of the window.
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    /// Set the width of the window.
    pub fn width(mut self, width: f32) -> Self {
        self.size.x = width;
        self
    }

    /// Set the height of the window.
    pub fn height(mut self, height: f32) -> Self {
        self.size.y = height;
        self
    }

    /// Set the window to be resizable or not.
    pub fn reziseable(mut self, reziseable: bool) -> Self {
        self.reziseable = reziseable;
        self
    }

    /// Set the clear color of the window.
    pub fn clear_color(mut self, color: Color) -> Self {
        self.clear_color = color;
        self
    }

    /// Set the icon of the window.
    pub fn icon(mut self, icon: impl Into<ImageSource>) -> Self {
        self.icon = Some(icon.into().load());
        self
    }

    pub fn open_parented(mut self, parent: &impl HasRawWindowHandle) {
        let options = baseview::WindowOpenOptions {
            title: self.title.clone(),
            size: Size::new(self.size.x as f64, self.size.y as f64),
            scale: WindowScalePolicy::ScaleFactor(1.0),
        };

        let event_sink = EventSink::new(AppEmitter::default());

        let builder = self.builder.take().unwrap();
        let event_callbacks = CallbackEmitter::new();
        let root = builder(&event_sink, &event_callbacks);

        Window::open_parented(parent, options, move |window| {
            #[cfg(feature = "wgpu")]
            let renderer = unsafe {
                ori_wgpu::WgpuRenderer::new(
                    &OldWindowHandle(window),
                    self.size.x as u32,
                    self.size.y as u32,
                )
            };

            let app_state_inner = AppStateInner {
                mouse_position: Vec2::ZERO,
                modifiers: Modifiers::default(),
                root,
                clear_color: self.clear_color,
                renderer,
            };

            let app_state = AppState {
                inner: Arc::new(Mutex::new(app_state_inner)),
            };

            app_state
        });
    }
}

#[derive(Default)]
struct AppEmitter {
    events: Vec<Event>,
    app_state: Option<AppState>,
}

impl EventEmitter for AppEmitter {
    fn send_event(&mut self, event: Event) {
        if let Some(task) = event.get::<Task>() {
            unsafe { task.poll() };
            return;
        }

        match self.app_state {
            Some(ref state) => {
                for event in self.events.drain(..) {
                    state.event(&event);
                }

                state.event(&event);
            }
            None => self.events.push(event),
        }
    }
}

struct AppStateInner {
    mouse_position: Vec2,
    modifiers: Modifiers,
    root: RootNode,
    clear_color: Color,
    #[cfg(feature = "wgpu")]
    renderer: ori_wgpu::WgpuRenderer,
}

#[derive(Clone)]
struct AppState {
    inner: Arc<Mutex<AppStateInner>>,
}

impl AppState {
    fn set_mouse_position(&self, position: Vec2) {
        let mut inner = self.inner.lock().unwrap();
        inner.mouse_position = position;
    }

    fn modifiers(&self) -> Modifiers {
        let inner = self.inner.lock().unwrap();
        inner.modifiers
    }

    fn mouse_position(&self) -> Vec2 {
        let inner = self.inner.lock().unwrap();
        inner.mouse_position
    }

    fn resize(&self, width: u32, heigth: u32) {
        {
            let inner = &mut *self.inner.lock().unwrap();

            inner.renderer.resize(width, heigth);
        }

        let size = Vec2::new(width as f32, heigth as f32);
        self.event(&Event::new(WindowResizeEvent::new(size)));
    }

    fn event(&self, event: &Event) {
        let inner = &mut *self.inner.lock().unwrap();

        inner.root.event(&inner.renderer, event);
    }

    fn draw(&self) {
        let inner = &mut *self.inner.lock().unwrap();

        inner.root.draw(&inner.renderer);

        #[cfg(feature = "wgpu")]
        (inner.renderer).render_frame(&inner.root.frame, inner.clear_color);
    }
}

impl WindowHandler for AppState {
    fn on_frame(&mut self, _window: &mut Window) {
        self.draw();
    }

    fn on_event(&mut self, _window: &mut Window, event: baseview::Event) -> EventStatus {
        match event {
            baseview::Event::Mouse(event) => match event {
                MouseEvent::CursorMoved {
                    position,
                    modifiers,
                } => {
                    let position = convert_point(position);
                    let modifiers = convert_modifiers(modifiers);

                    self.set_mouse_position(position);

                    let event = PointerEvent {
                        id: 0,
                        position,
                        modifiers,
                        ..Default::default()
                    };

                    self.event(&Event::new(event));

                    EventStatus::Captured
                }
                MouseEvent::ButtonPressed { button, modifiers } => {
                    let position = self.mouse_position();
                    let modifiers = convert_modifiers(modifiers);

                    let event = PointerEvent {
                        id: 0,
                        position,
                        modifiers,
                        button: Some(convert_mouse_button(button)),
                        pressed: true,
                        ..Default::default()
                    };

                    let event = Event::new(event);
                    self.event(&event);

                    EventStatus::Captured
                }
                MouseEvent::ButtonReleased { button, modifiers } => {
                    let position = self.mouse_position();
                    let modifiers = convert_modifiers(modifiers);

                    let event = PointerEvent {
                        id: 0,
                        position,
                        modifiers,
                        button: Some(convert_mouse_button(button)),
                        pressed: false,
                        ..Default::default()
                    };

                    let event = Event::new(event);
                    self.event(&event);

                    EventStatus::Captured
                }
                MouseEvent::WheelScrolled { delta, modifiers } => {
                    let position = self.mouse_position();
                    let modifiers = convert_modifiers(modifiers);

                    let event = PointerEvent {
                        id: 0,
                        position,
                        modifiers,
                        scroll_delta: convert_scroll_delta(delta),
                        ..Default::default()
                    };

                    let event = Event::new(event);
                    self.event(&event);

                    EventStatus::Captured
                }
                MouseEvent::CursorEntered => EventStatus::Captured,
                MouseEvent::CursorLeft => {
                    let position = self.mouse_position();
                    let modifiers = self.modifiers();

                    let event = PointerEvent {
                        id: 0,
                        position,
                        modifiers,
                        left: true,
                        ..Default::default()
                    };

                    let event = Event::new(event);
                    self.event(&event);

                    EventStatus::Captured
                }
            },
            baseview::Event::Keyboard(keyboard_types::KeyboardEvent {
                state,
                key,
                modifiers,
                ..
            }) => {
                let modifiers = convert_modifiers(modifiers);

                let event = KeyboardEvent {
                    key: convert_key(key),
                    modifiers,
                    pressed: is_pressed(state),
                    ..Default::default()
                };

                let event = Event::new(event);
                self.event(&event);

                EventStatus::Captured
            }
            baseview::Event::Window(event) => match event {
                baseview::WindowEvent::Resized(info) => {
                    self.resize(
                        info.logical_size().width as u32,
                        info.logical_size().height as u32,
                    );

                    EventStatus::Captured
                }
                baseview::WindowEvent::Focused => EventStatus::Captured,
                baseview::WindowEvent::Unfocused => EventStatus::Captured,
                baseview::WindowEvent::WillClose => EventStatus::Captured,
            },
        }
    }
}
