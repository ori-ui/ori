use ori_core::canvas::{Color, Scene, SceneRender};

use super::Render;

impl SceneRender for Render {
    fn render_scene(&mut self, scene: &mut Scene, clear_color: Color, width: u32, height: u32) {
        self.render_scene(scene, clear_color, width, height);
    }

    fn idle(&mut self) {
        self.clean();
    }
}
