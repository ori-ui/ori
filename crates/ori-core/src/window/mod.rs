mod backend;
mod descriptor;
mod id;
mod scope;

pub use backend::*;
pub use descriptor::*;
pub use id::*;
use ori_macro::view;
use ori_style::{StyleCache, StyleLoader};
pub use scope::*;

use glam::{UVec2, Vec2};
use ori_graphics::{Fonts, Frame, ImageCache, RenderBackend, Renderer};
use ori_reactive::{CallbackEmitter, Event, EventSink, Scope, Task};

use std::{collections::HashMap, fmt::Debug};

use crate::{
    Body, CloseWindow, Element, Key, KeyboardEvent, Modifiers, Node, OpenWindow, PointerButton,
    PointerEvent, RequestRedrawEvent, WindowClosedEvent, WindowResizedEvent,
};

const TEXT_FONT: &[u8] = include_bytes!("../../fonts/NotoSans-Medium.ttf");
const ICON_FONT: &[u8] = include_bytes!("../../fonts/MaterialIcons-Regular.ttf");

struct WindowUi<R: Renderer> {
    renderer: R,
    window: Window,
    element: Element,
    scope: Scope,
    event_sink: EventSink,
    event_emitter: CallbackEmitter<Event>,
    modifiers: Modifiers,
    pointers: HashMap<u64, Vec2>,
}

impl<R: Renderer> WindowUi<R> {
    fn update_window(&mut self, window_backend: &mut impl WindowBackend, window: &Window) {
        if self.window.title != window.title {
            self.window.title = window.title.clone();
            window_backend.set_title(window.id(), window.title.clone());
        }

        if self.window.resizable != window.resizable {
            self.window.resizable = window.resizable;
            window_backend.set_resizable(window.id(), window.resizable);
        }

        if self.window.clear_color != window.clear_color {
            self.window.clear_color = window.clear_color;
            window_backend.set_transparent(window.id(), window.clear_color.is_translucent());
        }

        if self.window.icon != window.icon {
            self.window.icon = window.icon.clone();
            window_backend.set_icon(window.id(), window.icon.clone());
        }

        if self.window.size != window.size {
            self.window.size = window.size;
            window_backend.set_size(window.id(), window.size);
        }

        if self.window.visible != window.visible {
            self.window.visible = window.visible;
            window_backend.set_visible(window.id(), window.visible);
        }

        if self.window.cursor != window.cursor {
            self.window.cursor = window.cursor;
            window_backend.set_cursor(window.id(), window.cursor);
        }
    }
}

/// An error that can occur when creating a window.
pub enum WindowError<W: WindowBackend, R: RenderBackend> {
    WindowBackend(W::Error),
    RenderBackend(R::Error),
}

impl<W: WindowBackend, R: RenderBackend> Debug for WindowError<W, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowError::WindowBackend(err) => write!(f, "WindowBackend({:?})", err),
            WindowError::RenderBackend(err) => write!(f, "RenderBackend({:?})", err),
        }
    }
}

/// The main entry point for the UI system.
///
/// When implementing a custom shell, this is primarily the type that you will interact with.
pub struct Windows<W, R>
where
    W: WindowBackend,
    R: RenderBackend<Surface = W::Surface>,
{
    /// The window backend, see [`WindowBackend`] for more information.
    pub window_backend: W,
    /// The render backend, see [`RenderBackend`] for more information.
    pub render_backend: R,
    /// The current frame, this is only stored here to avoid allocations.
    pub frame: Frame,
    /// The font system, see [`Fonts`] for more information.
    pub fonts: Fonts,
    /// The style cache, see [`StyleCache`] for more information.
    pub style_cache: StyleCache,
    /// The image cache, see [`ImageCache`] for more information.
    pub image_cache: ImageCache,
    /// The style loader, see [`StyleLoader`] for more information.
    pub style_loader: StyleLoader,

    window_ui: HashMap<WindowId, WindowUi<R::Renderer>>,
}

impl<W, R> Windows<W, R>
where
    W: WindowBackend,
    R: RenderBackend<Surface = W::Surface>,
{
    /// Creates a new [`Windows`] instance.
    ///
    /// **Note** that `W` and `R` need to have the same `Surface` type.
    pub fn new(window_backend: W, render_backend: R) -> Self {
        let mut fonts = Fonts::new();
        fonts.load_system_fonts();
        fonts.load_font_data(TEXT_FONT.to_vec());
        fonts.load_font_data(ICON_FONT.to_vec());

        Self {
            window_backend,
            render_backend,
            frame: Frame::new(),
            fonts,
            style_cache: StyleCache::new(),
            image_cache: ImageCache::new(),
            style_loader: StyleLoader::new(),
            window_ui: HashMap::new(),
        }
    }

    /// Returns the number of windows.
    pub fn len(&self) -> usize {
        self.window_ui.len()
    }

    /// Returns `true` if there are no windows.
    pub fn is_empty(&self) -> bool {
        self.window_ui.is_empty()
    }

    /// Returns the [`WindowId`]s of all windows.
    pub fn window_ids(&self) -> Vec<WindowId> {
        self.window_ui.keys().copied().collect()
    }

    /// Creates a new window.
    pub fn create_window(
        &mut self,
        target: W::Target<'_>,
        window: &Window,
        mut ui: impl FnMut(Scope) -> Node + Send + 'static,
    ) -> Result<(), WindowError<W, R>> {
        self.window_backend
            .create_window(target, window)
            .map_err(WindowError::WindowBackend)?;

        let surface = self
            .window_backend
            .create_surface(window.id())
            .map_err(WindowError::WindowBackend)?;

        let renderer = self
            .render_backend
            .create_renderer(surface, window.size.x, window.size.y)
            .map_err(WindowError::RenderBackend)?;

        let event_sink = self
            .window_backend
            .create_event_sink(window.id())
            .map_err(WindowError::WindowBackend)?;

        let event_emitter = CallbackEmitter::new();
        let scope = Scope::new(event_sink.clone(), event_emitter.clone());
        scope.with_context(scope.signal(window.clone()));

        let element = view! {scope,
            <Body>
                { ui(scope) }
            </Body>
        };

        let window_ui = WindowUi {
            renderer,
            window: window.clone(),
            element: element.into_element().unwrap(),
            scope,
            event_sink,
            event_emitter,
            modifiers: Modifiers::default(),
            pointers: HashMap::new(),
        };

        self.window_ui.insert(window.id(), window_ui);

        Ok(())
    }

    /// Close a window.
    pub fn close_window(&mut self, id: WindowId) {
        self.window_backend.close_window(id);

        if let Some(ui) = self.window_ui.remove(&id) {
            ui.scope.dispose();
        }

        for window in self.window_ids() {
            self.event_inner(window, &Event::new(WindowClosedEvent::new(id)));
        }
    }

    /// Run when the application is idle.
    ///
    /// This will reload styles if necessary, among other things.
    pub fn idle(&mut self) {
        self.image_cache.clean();

        match self.style_loader.reload() {
            Ok(true) => {
                self.style_cache.clear();

                for &ui in self.window_ui.keys() {
                    self.window_backend.request_redraw(ui);
                }
            }
            Err(err) => {
                eprintln!("Failed to reload styles: {}", err);
            }
            _ => {}
        }
    }

    /// Resize a window.
    pub fn resize_window(&mut self, id: WindowId, width: u32, height: u32) {
        if let Some(ui) = self.window_ui.get_mut(&id) {
            ui.scope.window().modify().size = UVec2::new(width, height);
            ui.renderer.resize(width, height);
        }

        let event = WindowResizedEvent::new(Vec2::new(width as f32, height as f32));
        self.event_inner(id, &Event::new(event));
    }

    /// Get the position of a pointer with a given `device` with a given `id`.
    pub fn get_pointer_position(&mut self, window: WindowId, device: u64) -> Option<Vec2> {
        let window = self.window_ui.get_mut(&window)?;
        Some(*window.pointers.entry(device).or_default())
    }

    /// Get the position of a pointer with a given `device` with a given `id`.
    ///
    /// If the pointer is not found, (0, 0) is returned.
    pub fn pointer_position(&mut self, window: WindowId, device: u64) -> Vec2 {
        (self.get_pointer_position(window, device)).unwrap_or_default()
    }

    /// Get the modifiers of a window.
    pub fn get_modfiers(&self, window: WindowId) -> Option<Modifiers> {
        Some(self.window_ui.get(&window)?.modifiers)
    }

    /// Get the modifiers of a window.
    ///
    /// If the window is not found, [`Modifiers::default()`] is returned.
    pub fn modifiers(&self, window: WindowId) -> Modifiers {
        self.get_modfiers(window).unwrap_or_default()
    }

    /// Handle a pointer being moved.
    pub fn pointer_moved(&mut self, window: WindowId, device: u64, position: Vec2) {
        if let Some(window) = self.window_ui.get_mut(&window) {
            window.pointers.insert(device, position);
        }

        let event = PointerEvent {
            device,
            position,
            modifiers: self.modifiers(window),
            ..Default::default()
        };

        self.event_inner(window, &Event::new(event));
    }

    /// Handle a pointer leaving the window.
    pub fn pointer_left(&mut self, window: WindowId, device: u64) {
        let event = PointerEvent {
            device,
            position: self.pointer_position(window, device),
            modifiers: self.modifiers(window),
            left: true,
            ..Default::default()
        };

        self.event_inner(window, &Event::new(event));
    }

    /// Handle a pointer button being pressed or released.
    pub fn pointer_button(
        &mut self,
        window: WindowId,
        device: u64,
        button: PointerButton,
        pressed: bool,
    ) {
        let event = PointerEvent {
            device,
            position: self.pointer_position(window, device),
            modifiers: self.modifiers(window),
            button: Some(button),
            pressed,
            ..Default::default()
        };

        self.event_inner(window, &Event::new(event));
    }

    /// Handle a pointer being scrolled.
    pub fn pointer_scroll(&mut self, window: WindowId, device: u64, delta: Vec2) {
        let event = PointerEvent {
            device,
            position: self.pointer_position(window, device),
            modifiers: self.modifiers(window),
            scroll_delta: delta,
            ..Default::default()
        };

        self.event_inner(window, &Event::new(event));
    }

    /// Handle a key being pressed or released.
    pub fn key(&mut self, window: WindowId, key: Key, pressed: bool) {
        let event = KeyboardEvent {
            key: Some(key),
            pressed,
            modifiers: self.modifiers(window),
            ..Default::default()
        };

        self.event_inner(window, &Event::new(event));
    }

    /// Handle text being input.
    pub fn text(&mut self, window: WindowId, text: String) {
        let event = KeyboardEvent {
            text: Some(text),
            modifiers: self.modifiers(window),
            ..Default::default()
        };

        self.event_inner(window, &Event::new(event));
    }

    /// Handle modifiers being changed.
    pub fn modifiers_changed(&mut self, window: WindowId, modifiers: Modifiers) {
        if let Some(window) = self.window_ui.get_mut(&window) {
            window.modifiers = modifiers;
        }

        let event = KeyboardEvent {
            modifiers,
            ..Default::default()
        };

        self.event_inner(window, &Event::new(event));
    }

    /// Handle an [`Event`].
    ///
    /// This should be called every time an [`Event`] is received from the [`EventSink`].
    pub fn event(&mut self, target: W::Target<'_>, id: WindowId, event: &Event) {
        if let Some(task) = event.get::<Task>() {
            task.poll();
            return;
        }

        if let Some(event) = event.get::<CloseWindow>() {
            match event.window {
                Some(id) => self.close_window(id),
                None => self.close_window(id),
            }

            return;
        }

        if let Some(event) = event.get::<OpenWindow>() {
            match self.create_window(target, event.window(), event.take_ui()) {
                Ok(_) => {
                    tracing::info!("Window opened");
                }
                Err(err) => {
                    tracing::error!("Failed to create window: {:?}", err);
                }
            }
        }

        if event.is::<RequestRedrawEvent>() {
            self.window_backend.request_redraw(id);
            return;
        }

        for id in self.window_ids() {
            self.event_inner(id, event);
        }
    }

    fn event_inner(&mut self, id: WindowId, event: &Event) {
        if let Some(ui) = self.window_ui.get_mut(&id) {
            ui.event_emitter.emit(event);

            let mut window = ui.scope.window().get();

            ori_reactive::effect::delay_effects(|| {
                ui.element.event_root_inner(
                    self.style_loader.stylesheet(),
                    &mut self.style_cache,
                    &ui.renderer,
                    &mut window,
                    &mut self.fonts,
                    &ui.event_sink,
                    event,
                    &mut self.image_cache,
                );
            });

            ui.update_window(&mut self.window_backend, &window);
        }
    }

    /// Layout a window.
    pub fn layout(&mut self, id: WindowId) {
        self.event_inner(id, &Event::new(()));

        if let Some(ui) = self.window_ui.get_mut(&id) {
            let mut window = ui.scope.window().get();

            ori_reactive::effect::delay_effects(|| {
                ui.element.layout_root_inner(
                    self.style_loader.stylesheet(),
                    &mut self.style_cache,
                    &ui.renderer,
                    &mut window,
                    &mut self.fonts,
                    &ui.event_sink,
                    &mut self.image_cache,
                );
            });

            ui.update_window(&mut self.window_backend, &window);
        }
    }

    /// Draw a window.
    pub fn draw(&mut self, id: WindowId) {
        self.layout(id);

        if let Some(ui) = self.window_ui.get_mut(&id) {
            self.frame.clear();

            let mut window = ui.scope.window().get();

            ori_reactive::effect::delay_effects(|| {
                ui.element.draw_root_inner(
                    self.style_loader.stylesheet(),
                    &mut self.style_cache,
                    &mut self.frame,
                    &ui.renderer,
                    &mut window,
                    &mut self.fonts,
                    &ui.event_sink,
                    &mut self.image_cache,
                );
            });

            ui.update_window(&mut self.window_backend, &window);

            let clear_color = window.clear_color;
            (ui.renderer).render_frame(&self.frame, clear_color);
        }
    }

    pub fn style_loader(&self) -> &StyleLoader {
        &self.style_loader
    }
}
