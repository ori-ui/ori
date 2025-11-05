use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "layer-shell")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Layer {
    Background,
    Bottom,
    Top,
    Overlay,
}

#[cfg(feature = "layer-shell")]
impl From<Layer> for gtk4_layer_shell::Layer {
    fn from(layer: Layer) -> Self {
        match layer {
            Layer::Background => gtk4_layer_shell::Layer::Background,
            Layer::Bottom => gtk4_layer_shell::Layer::Bottom,
            Layer::Top => gtk4_layer_shell::Layer::Top,
            Layer::Overlay => gtk4_layer_shell::Layer::Overlay,
        }
    }
}

#[derive(Debug)]
pub struct Window {
    pub(crate) id: u64,
    pub(crate) title: String,
    pub(crate) width: u32,
    pub(crate) height: u32,

    #[cfg(feature = "layer-shell")]
    pub(crate) layer: Option<Layer>,
    #[cfg(feature = "layer-shell")]
    pub(crate) exclusive_zone: Option<i32>,
    #[cfg(feature = "layer-shell")]
    pub(crate) margin_top: i32,
    #[cfg(feature = "layer-shell")]
    pub(crate) margin_right: i32,
    #[cfg(feature = "layer-shell")]
    pub(crate) margin_bottom: i32,
    #[cfg(feature = "layer-shell")]
    pub(crate) margin_left: i32,
    #[cfg(feature = "layer-shell")]
    pub(crate) anchor_top: bool,
    #[cfg(feature = "layer-shell")]
    pub(crate) anchor_right: bool,
    #[cfg(feature = "layer-shell")]
    pub(crate) anchor_bottom: bool,
    #[cfg(feature = "layer-shell")]
    pub(crate) anchor_left: bool,
}

impl Window {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        Self {
            id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
            title: String::from("Ori Gtk4 App"),
            width: 800,
            height: 600,

            #[cfg(feature = "layer-shell")]
            layer: None,
            #[cfg(feature = "layer-shell")]
            exclusive_zone: None,
            #[cfg(feature = "layer-shell")]
            margin_top: 0,
            #[cfg(feature = "layer-shell")]
            margin_right: 0,
            #[cfg(feature = "layer-shell")]
            margin_bottom: 0,
            #[cfg(feature = "layer-shell")]
            margin_left: 0,
            #[cfg(feature = "layer-shell")]
            anchor_top: false,
            #[cfg(feature = "layer-shell")]
            anchor_right: false,
            #[cfg(feature = "layer-shell")]
            anchor_bottom: false,
            #[cfg(feature = "layer-shell")]
            anchor_left: false,
        }
    }

    pub fn title(mut self, title: impl ToString) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    pub fn size(self, width: u32, height: u32) -> Self {
        self.width(width).height(height)
    }

    #[cfg(feature = "layer-shell")]
    pub fn layer(mut self, layer: impl Into<Option<Layer>>) -> Self {
        self.layer = layer.into();
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn exclusive_zone(mut self, zone: impl Into<Option<i32>>) -> Self {
        self.exclusive_zone = zone.into();
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn margin_top(mut self, margin: i32) -> Self {
        self.margin_top = margin;
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn margin_right(mut self, margin: i32) -> Self {
        self.margin_right = margin;
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn margin_bottom(mut self, margin: i32) -> Self {
        self.margin_bottom = margin;
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn margin_left(mut self, margin: i32) -> Self {
        self.margin_left = margin;
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn anchor_top(mut self, anchor: bool) -> Self {
        self.anchor_top = anchor;
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn anchor_right(mut self, anchor: bool) -> Self {
        self.anchor_right = anchor;
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn anchor_bottom(mut self, anchor: bool) -> Self {
        self.anchor_bottom = anchor;
        self
    }

    #[cfg(feature = "layer-shell")]
    pub fn anchor_left(mut self, anchor: bool) -> Self {
        self.anchor_left = anchor;
        self
    }
}
