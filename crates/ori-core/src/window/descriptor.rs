use crate::{Size, WindowId};

#[derive(Clone, Debug, PartialEq)]
pub struct WindowDescriptor {
    pub id: WindowId,
    pub title: String,
    pub size: Size,
    pub resizable: bool,
    pub decorated: bool,
    pub transparent: bool,
    pub maximized: bool,
    pub visible: bool,
}

impl Default for WindowDescriptor {
    fn default() -> Self {
        Self {
            id: WindowId::next(),
            title: String::from("Ori App"),
            size: Size::new(800.0, 600.0),
            resizable: true,
            decorated: true,
            transparent: false,
            maximized: false,
            visible: true,
        }
    }
}
