use event::AppEvent;

use crate::event::{self, EventListener};

pub struct DebugLogger;

impl EventListener for DebugLogger {
    fn on_event(&mut self, event: &AppEvent) {
        let lvl = if matches!(event, &AppEvent::Err(_)) {
            log::Level::Error
        } else {
            log::Level::Debug
        };

        log::log!(lvl, "{:?}", event);
    }
}
