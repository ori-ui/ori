use ori_graphics::{Frame, Renderer};
use ori_reactive::{CallbackEmitter, Event, EventSink};

use crate::{Cursor, ImageCache, Node, RequestRedrawEvent, StyleCache, StyleLoader};

/// The root node of the application.
pub struct RootNode {
    pub style_loader: StyleLoader,
    pub event_sink: EventSink,
    pub event_callbacks: CallbackEmitter<Event>,
    pub style_cache: StyleCache,
    pub image_cache: ImageCache,
    pub cursor: Cursor,
    pub frame: Frame,
    pub node: Node,
}

impl RootNode {
    pub fn new(node: Node, event_sink: EventSink, event_callbacks: CallbackEmitter<Event>) -> Self {
        Self {
            style_loader: StyleLoader::new(),
            event_sink,
            event_callbacks,
            style_cache: StyleCache::new(),
            image_cache: ImageCache::new(),
            cursor: Cursor::default(),
            frame: Frame::new(),
            node,
        }
    }

    pub fn with_style_loader(mut self, style_loader: StyleLoader) -> Self {
        self.style_loader = style_loader;
        self
    }

    pub fn idle(&mut self) {
        self.reload_style();
    }

    pub fn reload_style(&mut self) {
        match self.style_loader.reload() {
            Ok(reload) if reload => {
                tracing::info!("Reloaded style");
                self.style_cache.clear();
                self.event_sink.emit(RequestRedrawEvent);
            }
            Err(err) => {
                tracing::error!("Failed to reload style: {}", err);
            }
            _ => {}
        }
    }

    pub fn clean(&mut self) {
        self.image_cache.clean();
    }

    pub fn event(&mut self, renderer: &dyn Renderer, event: &Event) {
        self.event_callbacks.emit(event);

        self.cursor = Cursor::default();

        ori_reactive::delay_effects(|| {
            self.node.event_root_inner(
                &self.style_loader.stylesheet(),
                &mut self.style_cache,
                renderer,
                &self.event_sink,
                event,
                &mut self.image_cache,
                &mut self.cursor,
            );
        });

        self.clean();
    }

    pub fn layout(&mut self, renderer: &dyn Renderer) {
        self.cursor = Cursor::default();

        self.event(renderer, &Event::new(()));

        ori_reactive::delay_effects(|| {
            self.node.layout_root_inner(
                &self.style_loader.stylesheet(),
                &mut self.style_cache,
                renderer,
                &self.event_sink,
                &mut self.image_cache,
                &mut self.cursor,
            );
        });

        self.clean();
    }

    pub fn draw(&mut self, renderer: &dyn Renderer) {
        self.layout(renderer);

        self.cursor = Cursor::default();
        self.frame.clear();

        ori_reactive::delay_effects(|| {
            self.node.draw_root_inner(
                &self.style_loader.stylesheet(),
                &mut self.style_cache,
                &mut self.frame,
                renderer,
                &self.event_sink,
                &mut self.image_cache,
                &mut self.cursor,
            );
        });

        self.clean();
    }
}
