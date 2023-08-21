use std::sync::Mutex;

use ori_core::proxy::ProxyWaker;
use winit::event_loop::EventLoopProxy;

pub(crate) struct WinitWaker {
    pub(crate) proxy: Mutex<EventLoopProxy<()>>,
}

impl ProxyWaker for WinitWaker {
    fn wake(&self) {
        if let Ok(proxy) = self.proxy.lock() {
            proxy.send_event(()).ok();
        }
    }
}
