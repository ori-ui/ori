use std::sync::Mutex;

use ori_core::{Element, RootElement, WindowId};
use ori_graphics::{prelude::Vec2, Color, ImageData};
use ori_reactive::{CallbackEmitter, Event, EventSink, Scope};
use winit::{
    dpi::LogicalSize,
    window::{Icon, WindowBuilder},
};

type Builder = Box<dyn FnOnce(&EventSink, &CallbackEmitter<Event>) -> RootElement + Send + Sync>;

pub struct OpenWindowEvent {
    pub(crate) title: String,
    pub(crate) size: Vec2,
    pub(crate) resizable: bool,
    pub(crate) clear_color: Color,
    pub(crate) icon: Option<ImageData>,
    pub(crate) id: WindowId,
    pub(crate) builder: Mutex<Option<Builder>>,
}

impl OpenWindowEvent {
    pub fn new(content: impl FnOnce(Scope) -> Element + Send + Sync + 'static) -> Self {
        let builder = Box::new(
            move |event_sink: &EventSink, event_callback: &CallbackEmitter<Event>| {
                let scope = Scope::new(event_sink.clone(), event_callback.clone());
                RootElement::new(content(scope), event_sink.clone(), event_callback.clone())
            },
        );

        Self {
            title: String::from("Ori App"),
            size: Vec2::new(800.0, 600.0),
            resizable: true,
            clear_color: Color::WHITE,
            icon: None,
            id: WindowId::new(),
            builder: Mutex::new(Some(builder)),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.size.x = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.size.y = height;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn clear_color(mut self, clear_color: Color) -> Self {
        self.clear_color = clear_color;
        self
    }

    pub fn transparent(mut self) -> Self {
        self.clear_color = Color::TRANSPARENT;
        self
    }

    pub fn id(&self) -> WindowId {
        self.id
    }

    pub fn open(self, cx: Scope) -> WindowId {
        let id = self.id;
        cx.emit_event(self);
        id
    }

    pub(crate) fn window_builder(&self) -> WindowBuilder {
        let size = LogicalSize::new(self.size.x, self.size.y);

        let icon = match self.icon {
            Some(ref icon) => {
                let pixels = icon.pixels().to_vec();
                Icon::from_rgba(pixels, icon.width(), icon.height()).ok()
            }
            None => None,
        };

        let builder = WindowBuilder::new()
            .with_title(&self.title)
            .with_inner_size(size)
            .with_resizable(self.resizable)
            .with_transparent(self.clear_color.is_translucent())
            .with_window_icon(icon);

        builder
    }

    pub(crate) fn builder(&self) -> Builder {
        let mut builder = self.builder.lock().unwrap();
        builder.take().unwrap()
    }
}
