use clipboard::ClipboardProvider as _;
use ori_core::clipboard::ClipboardProvider;

pub(crate) struct WinitClipboardProvider {
    inner: clipboard::ClipboardContext,
}

impl WinitClipboardProvider {
    pub(crate) fn new() -> Self {
        Self {
            inner: clipboard::ClipboardContext::new().unwrap(),
        }
    }
}

impl ClipboardProvider for WinitClipboardProvider {
    fn get(&mut self) -> String {
        self.inner.get_contents().unwrap_or_default()
    }

    fn set(&mut self, contents: String) {
        self.inner.set_contents(contents).unwrap();
    }
}
