use ori_core::{Size, UiBuilder, View, WindowDescriptor};

use crate::Error;

pub struct App<T> {
    pub(crate) window: WindowDescriptor,
    pub(crate) builder: UiBuilder<T>,
    pub(crate) data: T,
}

impl<T: 'static> App<T> {
    pub fn new<V>(mut builder: impl FnMut(&mut T) -> V + 'static, data: T) -> Self
    where
        V: View<T> + 'static,
        V::State: 'static,
    {
        Self {
            window: WindowDescriptor::default(),
            builder: Box::new(move |data| Box::new(builder(data))),
            data,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.window.title = title.into();
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.window.size = Size::new(width, height);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.window.size.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.window.size.height = height;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.window.resizable = resizable;
        self
    }

    pub fn decorated(mut self, decorated: bool) -> Self {
        self.window.decorated = decorated;
        self
    }

    pub fn transparent(mut self, transparent: bool) -> Self {
        self.window.transparent = transparent;
        self
    }

    pub fn maximized(mut self, maximized: bool) -> Self {
        self.window.maximized = maximized;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.window.visible = visible;
        self
    }

    pub fn try_run(self) -> Result<(), Error> {
        crate::run::run(self)
    }

    pub fn run(self) {
        if let Err(err) = self.try_run() {
            panic!("{}", err);
        }
    }
}
