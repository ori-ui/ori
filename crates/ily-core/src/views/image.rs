use glam::Vec2;
use ily_graphics::{ImageHandle, ImageSource, Mesh};
use ily_macro::Build;

use crate::{
    BoxConstraints, Context, DrawContext, Event, EventContext, LayoutContext, Style, View,
};

#[derive(Clone, Default, Debug, Build)]
pub struct Image {
    #[prop]
    src: ImageSource,
}

impl Image {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ImageState {
    src: ImageSource,
    handle: Option<ImageHandle>,
}

impl ImageState {
    pub fn update(&mut self, context: &mut impl Context, src: &ImageSource) -> &ImageHandle {
        if self.src != *src || self.handle.is_none() {
            self.src = src.clone();
            self.handle = Some(context.load_image(src));
        }

        self.handle.as_ref().unwrap()
    }
}

impl View for Image {
    type State = ImageState;

    fn build(&self) -> Self::State {
        Default::default()
    }

    fn style(&self) -> Style {
        Style::new("image")
    }

    fn event(&self, _state: &mut Self::State, _cx: &mut EventContext, _event: &Event) {}

    fn layout(&self, state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        let min_width = cx.style_range_group("min-width", "width", bc.width());
        let max_width = cx.style_range_group("max-width", "width", bc.width());

        let min_height = cx.style_range_group("min-width", "height", bc.height());
        let max_height = cx.style_range_group("min-height", "height", bc.height());

        let min_size = bc.constrain(Vec2::new(min_width, min_height));
        let max_size = bc.constrain(Vec2::new(max_width, max_height));

        let handle = state.update(cx, &self.src);

        // try to fit the image in the min/max size
        // while maintaining the aspect ratio
        let mut size = handle.size();
        let aspect = size.x / size.y;

        if size.x > max_size.x {
            size.x = max_size.x;
            size.y = size.x / aspect;
        }

        if size.y > max_size.y {
            size.y = max_size.y;
            size.x = size.y * aspect;
        }

        if size.x < min_size.x {
            size.x = min_size.x;
            size.y = size.x / aspect;
        }

        if size.y < min_size.y {
            size.y = min_size.y;
            size.x = size.y * aspect;
        }

        size
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        let handle = state.update(cx, &self.src);
        let mesh = Mesh::image(cx.rect(), handle.clone());
        cx.draw_primitive(mesh);
    }
}
