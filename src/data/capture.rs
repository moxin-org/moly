use makepad_widgets::Cx;

use crate::capture::{CaptureHandler, Error, Event};

#[derive(Debug, Default, Clone)]
pub enum CaptureAction {
    #[default]
    None,
    Capture {
        event: Event,
    },
    Error {
        #[expect(unused)]
        error: Error,
    },
}

struct CaptureManager;

impl CaptureManager {}

impl CaptureHandler for CaptureManager {
    fn capture(&self, event: Event) {
        eprintln!("capture: {event:?}");
        Cx::post_action(CaptureAction::Capture { event });
    }

    fn error(&self, error: Error) {
        eprintln!("capture error: {error:?}");
        Cx::post_action(CaptureAction::Error { error });
    }
}

pub fn register_capture_manager() {
    crate::capture::register_handler(CaptureManager).expect("Failed to register capture manager");
}
