use arboard::Clipboard as Arboard;
use fission_core::env::Clipboard;
use std::sync::{Arc, Mutex};

pub struct DesktopClipboard {
    inner: Arc<Mutex<Option<Arboard>>>,
}

impl DesktopClipboard {
    pub fn new() -> Self {
        let cb = Arboard::new().ok();
        Self {
            inner: Arc::new(Mutex::new(cb)),
        }
    }
}

impl Clipboard for DesktopClipboard {
    fn get_text(&self) -> Option<String> {
        if let Ok(mut lock) = self.inner.lock() {
            if let Some(cb) = lock.as_mut() {
                return cb.get_text().ok();
            }
        }
        None
    }

    fn set_text(&self, text: &str) {
        if let Ok(mut lock) = self.inner.lock() {
            if let Some(cb) = lock.as_mut() {
                let _ = cb.set_text(text);
            }
        }
    }
}
