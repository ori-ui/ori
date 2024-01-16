use clipboard::ClipboardProvider as _;
use ori_core::clipboard::Clipboard;

pub(crate) struct WinitClipboard {
    inner: clipboard::ClipboardContext,
}

impl WinitClipboard {
    pub(crate) fn new() -> Self {
        Self {
            inner: clipboard::ClipboardContext::new().unwrap(),
        }
    }
}

impl Clipboard for WinitClipboard {
    fn get(&mut self) -> String {
        self.inner.get_contents().unwrap_or_default()
    }

    fn set(&mut self, contents: String) {
        self.inner.set_contents(contents).unwrap();
    }
}
