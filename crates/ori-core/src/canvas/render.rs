use super::{Color, Scene};

/// Trait for rendering a scene.
pub trait SceneRender {
    /// Render a scene.
    fn render_scene(&mut self, scene: &mut Scene, clear_color: Color, width: u32, height: u32);

    /// Called when the application is idle.
    ///
    /// This should be used to clean up any resources that are no longer needed.
    fn idle(&mut self) {}
}
