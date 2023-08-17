use crate::Scene;

/// Trait for rendering a scene.
pub trait SceneRender {
    /// Render a scene.
    fn render_scene(&mut self, scene: &mut Scene, width: u32, height: u32);
}
