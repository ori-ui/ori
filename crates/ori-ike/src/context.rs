use winit::window::WindowAttributes;

pub struct Context {
    pub(crate) windows: Windows,
    pub(crate) commands: Vec<Command>,
}

#[derive(Debug)]
pub(crate) enum Command {
    CreateWindow {
        key: ori::Key,
        attributes: WindowAttributes,
    },
}
