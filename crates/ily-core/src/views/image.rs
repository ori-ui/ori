use glam::Vec2;
use ily_graphics::{ImageData, ImageHandle, Mesh, Renderer};
use ily_macro::Build;

use crate::{
    BoxConstraints, Context, DrawContext, Event, EventContext, LayoutContext, Style, View,
};

#[derive(Clone, Default, Debug, Build)]
pub struct Image {
    #[prop]
    src: ImageData,
}

impl Image {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ImageState {
    handle: Option<(ImageData, ImageHandle)>,
}

impl ImageState {
    pub fn update(&mut self, renderer: &dyn Renderer, data: ImageData) {
        if let Some((old_data, _)) = self.handle.as_ref() {
            if *old_data != data {
                let handle = renderer.create_image(&data);
                self.handle = Some((data, handle));
            }
        } else {
            let handle = renderer.create_image(&data);
            self.handle = Some((data, handle));
        }
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

    fn layout(&self, _state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        let min_width = cx.style_range_or("width", "min-width", bc.width());
        let max_width = cx.style_range_or("width", "max-width", bc.width());

        let min_height = cx.style_range_or("height", "min-height", bc.height());
        let max_height = cx.style_range_or("height", "max-height", bc.height());

        let min_size = bc.constrain(Vec2::new(min_width, min_height));
        let max_size = bc.constrain(Vec2::new(max_width, max_height));

        // try to fit the image in the min/max size
        // while maintaining the aspect ratio
        let mut size = self.src.size();
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
        state.update(cx.renderer, self.src.clone());

        let (_, handle) = state.handle.as_ref().unwrap();

        let mesh = Mesh::image(cx.rect(), handle.clone());
        cx.draw_primitive(mesh);
    }
}
