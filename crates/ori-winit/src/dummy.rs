use ori_core::canvas::{Color, Scene, SceneRender};

use crate::log::warn_internal;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DummyRender;

impl SceneRender for DummyRender {
    fn render_scene(&mut self, _scene: &mut Scene, _clear_color: Color, _width: u32, _height: u32) {
        warn_internal!("No renderer available. Try using the `wgpu` feature.");
    }
}
