use glam::Vec2;
use ori_graphics::{ImageHandle, ImageSource, Mesh};
use ori_macro::Build;

use crate::{BoxConstraints, Context, DrawContext, LayoutContext, Style, View};

/// A view that displays an image.
#[derive(Clone, Default, Debug, Build)]
pub struct Image {
    /// The source of the image.
    #[prop]
    pub src: ImageSource,
}

impl Image {
    /// Creates a new image view.
    pub fn new() -> Self {
        Self::default()
    }
}

/// The state of an image view.
#[derive(Clone, Debug, Default)]
pub struct ImageState {
    src: ImageSource,
    handle: Option<ImageHandle>,
}

impl ImageState {
    /// Updates the image handle if the source has changed.
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

    #[tracing::instrument(name = "Image", skip(self, state, cx, bc))]
    fn layout(&self, state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        let handle = state.update(cx, &self.src);

        // try to fit the image in the min/max size
        // while maintaining the aspect ratio
        let mut size = handle.size();
        let aspect = size.x / size.y;

        if size.x > bc.max.x {
            size.x = bc.max.x;
            size.y = size.x / aspect;
        }

        if size.y > bc.max.y {
            size.y = bc.max.y;
            size.x = size.y * aspect;
        }

        if size.x < bc.min.x {
            size.x = bc.min.x;
            size.y = size.x / aspect;
        }

        if size.y < bc.min.y {
            size.y = bc.min.y;
            size.x = size.y * aspect;
        }

        size
    }

    #[tracing::instrument(name = "Image", skip(self, state, cx))]
    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        let handle = state.update(cx, &self.src);
        let mesh = Mesh::image(cx.rect(), handle.clone());
        cx.draw(mesh);
    }
}
