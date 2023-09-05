use std::sync::Mutex;

use ori_core::command::EventLoopWaker;
use winit::event_loop::EventLoopProxy;

pub(crate) struct WinitWaker {
    pub(crate) proxy: Mutex<EventLoopProxy<()>>,
}

impl EventLoopWaker for WinitWaker {
    fn wake(&self) {
        if let Ok(proxy) = self.proxy.lock() {
            proxy.send_event(()).ok();
        }
    }
}
