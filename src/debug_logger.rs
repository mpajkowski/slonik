use event::AppEvent;

use crate::event::{self, EventListener};

pub struct DebugLogger;

impl EventListener for DebugLogger {
    fn on_event(&mut self, event: &AppEvent) {
        if let AppEvent::Err(why) = event {
            log::error!("{}", why);
        }
    }
}
