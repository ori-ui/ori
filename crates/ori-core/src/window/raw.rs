use std::fmt::Debug;

use crate::Image;

pub trait RawWindow {
    fn title(&self) -> String;
    fn set_title(&mut self, title: &str);

    fn set_icon(&mut self, icon: Option<&Image>);

    fn size(&self) -> (u32, u32);
    fn set_size(&mut self, width: u32, height: u32);

    fn resizable(&self) -> bool;
    fn set_resizable(&mut self, resizable: bool);

    fn decorated(&self) -> bool;
    fn set_decorated(&mut self, decorated: bool);

    fn scale_factor(&self) -> f32;

    fn minimized(&self) -> bool;
    fn set_minimized(&mut self, minimized: bool);

    fn maximized(&self) -> bool;
    fn set_maximized(&mut self, maximized: bool);

    fn visible(&self) -> bool;
    fn set_visible(&mut self, visible: bool);

    fn request_draw(&mut self);
}

impl Debug for dyn RawWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawWindow")
            .field("title", &self.title())
            .field("size", &self.size())
            .finish()
    }
}
